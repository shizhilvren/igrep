use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info, trace, warn};
use std::{fs, io::BufRead, ops::Mul, path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::lsp::{self, builder::Builder, data::HoversData, index::FileIndex};

fn init_lsp_client(
    rt: &tokio::runtime::Runtime,
    log_file: String,
    compile_commands_dir: String,
    debug: bool,
) -> Result<crate::clang::lsp_server_wraper::Client> {
    let handle: tokio::task::JoinHandle<
        Result<crate::clang::lsp_server_wraper::Client, anyhow::Error>,
    > = rt.spawn(async move {
        let client_wrapper = crate::clang::lsp_server_wraper::ClangdClient::new(
            &log_file,
            compile_commands_dir,
            debug,
        )?;
        let client_to_request_sender = client_wrapper.warpper_loop().await?;
        let rec = client_to_request_sender.initialize()?;
        let _ = rec
            .await
            .map_err(|e| anyhow!("get init response fail: {}", e))?;
        client_to_request_sender.initialized()?;
        info!("LSP initalized");
        Ok(client_to_request_sender)
    });

    let client_to_request_sender = rt
        .block_on(handle)
        .map_err(|e| anyhow!("init lsp task join fail: {}", e))??;
    Ok(client_to_request_sender)
}

fn wait_index_done(
    rt: &tokio::runtime::Runtime,
    client_to_request_sender: crate::clang::lsp_server_wraper::Client,
    file_index_data_builder: lsp::builder::FileIndexDataBuilder,
) -> Result<(
    crate::clang::lsp_server_wraper::Client,
    lsp::builder::FileIndexDataBuilder,
)> {
    let handle = rt.spawn(async move {
        file_index_data_builder
            .file_builders()
            .iter()
            .next()
            .map_or(Ok::<(), anyhow::Error>(()), |file_builder| {
                let file_path = file_builder
                    .file_index()
                    .path()
                    .to_string_lossy()
                    .to_string();
                let file_data = file_builder.file_data();
                client_to_request_sender.did_open(&file_path, file_data.lines())?;
                client_to_request_sender.did_close(&file_path)?;
                Ok(())
            })?;

        let mut client_to_request_sender = client_to_request_sender;
        client_to_request_sender.index_done().await?;
        Ok::<_, anyhow::Error>((client_to_request_sender, file_index_data_builder))
    });

    rt.block_on(handle)
        .map_err(|e| anyhow!("wait index done task join fail: {}", e))?
}

async fn fetch_file_semantic_tokens(
    client: &mut crate::clang::lsp_server_wraper::Client,
    file_path: &str,
) -> Result<lsp_types::SemanticTokens> {
    let semantic_tokens_response = client.semantic_tokens_full(file_path)?;
    let semantic_tokens_response = semantic_tokens_response
        .await
        .map_err(|e| anyhow!("semantic token response recv fail: {}", e))?;
    debug!("获取语义标记成功: {}", file_path);
    let semantic_tokens: lsp_types::SemanticTokens =
        serde_json::from_value(semantic_tokens_response.val)?;
    Ok(semantic_tokens)
}

pub async fn handle_semantics(
    client: &mut crate::clang::lsp_server_wraper::Client,
    file_path: &str,
    tokens: &lsp_types::SemanticTokens,
) -> Result<HoversData> {
    let ans = tokens
        .data
        .iter()
        .scan((0, 0), |(row, col), token| {
            match token.delta_line {
                0 => {
                    *col += token.delta_start;
                }
                _ => {
                    *row += token.delta_line;
                    *col = token.delta_start;
                }
            }
            Some((row.clone(), col.clone()))
        })
        .collect::<Vec<_>>();
    let mut hovers = Vec::new();
    for (row, col) in ans {
        debug!("请求悬停信息: {}:{}:{}", file_path, row, col);
        let hover_response = client
            .hover(file_path, row, col)?
            .await
            .map_err(|e| anyhow!("hover response recv fail: {}", e))?;
        let hover = serde_json::from_value::<lsp_types::Hover>(hover_response.val);
        debug!("悬停信息获取成功: {}:{}:{}", file_path, row, col);
        match hover {
            Ok(hover) => {
                hovers.push(hover);
            }
            Err(e) => {
                trace!("Failed to parse hover response: {}", e);
            }
        }
    }
    HoversData::try_from(hovers)
}

pub fn main(
    file_list: &str,
    debug: bool,
    log_file: String,
    compile_commands_dir: String,
    config: &str,
) -> Result<()> {
    let worker_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .saturating_mul(1);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .thread_name("lsp-server-wrapper") // 设置线程名称
        .enable_all() // 启用 IO 和 Time 驱动
        .build()
        .unwrap();
    let client_to_request_sender = init_lsp_client(&rt, log_file, compile_commands_dir, debug)?;

    let file_content = fs::read(file_list)?;
    let files_list: Vec<String> = std::io::BufReader::new(&file_content[..])
        .lines()
        .filter_map(Result::ok)
        .filter(|file| !file.is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    let mut file_index_builder = lsp::builder::FileIndexBuilder::from(());
    files_list.into_iter().try_for_each(|file_name| {
        let file_index = FileIndex::from(file_name);
        file_index_builder.insert(file_index)?;
        Ok::<(), anyhow::Error>(())
    })?;
    let file_index_data_builder = lsp::builder::FileIndexDataBuilder::try_from(file_index_builder)?;

    let (client_to_request_sender, file_index_data_builder) =
        wait_index_done(&rt, client_to_request_sender, file_index_data_builder)?;
    info!("index done");

    let semantic_token = rt.block_on(async {
        let mut join_set = JoinSet::new();
        let total = file_index_data_builder.file_builders().len();
        let semaphore = Arc::new(Semaphore::new(worker_threads.mul(2).max(1).min(9999)));
        let progress_bar = ProgressBar::new(total as u64);
        progress_bar.set_message("semantic tokens");
        if let Ok(style) = ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        ) {
            progress_bar.set_style(style.progress_chars("=> "));
        }
        progress_bar.inc(0);
        file_index_data_builder
            .file_builders()
            .iter()
            .for_each(|file_builder| {
                let mut client = client_to_request_sender.clone();
                let semaphore = Arc::clone(&semaphore);
                let file_index = file_builder.file_index().clone();
                let file_path = file_builder
                    .file_index()
                    .path()
                    .to_string_lossy()
                    .to_string();
                let hover_file_path = file_path.clone();
                let file_lines = file_builder.file_data().lines().to_vec();

                join_set.spawn(async move {
                    let _open_span_permit = semaphore
                        .acquire_owned()
                        .await
                        .map_err(|e| anyhow!("semaphore acquire fail: {}", e))?;
                    trace!("start to get semantic tokens: {}", file_path);
                    client.did_open(&file_path, &file_lines)?;
                    let semantic_tokens =
                        fetch_file_semantic_tokens(&mut client, &file_path).await?;
                    trace!("semantic tokens get finish: {}", file_path);
                    let hovers =
                        handle_semantics(&mut client, &hover_file_path, &semantic_tokens).await?;
                    client.did_close(&file_path)?;
                    trace!("semantic tokens and hovers get finish: {}", file_path);
                    let semantic_tokens_data =
                        lsp::data::FileSemanticTokensData::from(semantic_tokens);

                    Ok((file_index, semantic_tokens_data, hovers))
                });
            });

        let mut semantic_token = Vec::with_capacity(total);
        while let Some(task_result) = join_set.join_next().await {
            let task_result: Result<_, anyhow::Error> =
                task_result.map_err(|e| anyhow!("semantic token task join fail: {}", e))?;
            let (file_index, semantic_tokens_data, hovers) = task_result?;
            progress_bar.set_message(file_index.path().to_string_lossy().to_string());
            semantic_token.push((file_index, semantic_tokens_data, hovers));
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("semantic tokens done");
        Ok::<Vec<_>, anyhow::Error>(semantic_token)
    })?;
    info!("all semantic tokens get finish.");

    if debug {
        semantic_token
            .iter()
            .try_for_each(|(file_index, semantic_tokens, hovers)| {
                info!(
                    "文件: {:?}, 语义标记数量: {} , 悬停信息数量: {} 总字节数: {}",
                    file_index.path(),
                    semantic_tokens.tokens().len(),
                    hovers.hovers().len(),
                    hovers.hovers().iter().map(|t| t.hover().len() as u64).sum::<u64>()
                );
                Ok::<(), anyhow::Error>(())
            })?;
    }

    let builder = Builder::try_from((
        file_index_data_builder,
        semantic_token
            .into_iter()
            .map(|(file_index, semantic_tokens, hovers)| (file_index, semantic_tokens))
            .collect(),
    ))?;
    builder.dump(PathBuf::from(config).as_path())?;

    // files_list.into_iter().try_for_each(|file| {
    //     client.open_file(&file)?;
    //     match client.get_semantic_tokens_full(&file) {
    //         Ok(tokens) => {
    //             let tokens: lsp_types::SemanticTokens = serde_json::from_value(tokens)?;
    //             info!("{:?}", tokens);
    //             client.handle_semantics(&file, tokens)?;
    //         }
    //         Err(e) => error!("获取语义标记时出错: {}", e),
    //     }
    //     client.did_close(&file)?;
    //     Ok::<(), anyhow::Error>(())
    // })?;

    // println!("获取位置 {}:{} 的信息...", line, column);

    // // 获取悬停信息
    // println!("\n=== 悬停信息 ===");
    // match client.get_hover(file_path, line, column) {
    //     Ok(hover) => println!("{}", serde_json::to_string_pretty(&hover)?),
    //     Err(e) => println!("获取悬停信息时出错: {}", e),
    // }

    // match client.get_code_lens(file_path, line, column) {
    //     Ok(codelens) => println!("{}", serde_json::to_string_pretty(&codelens)?),
    //     Err(e) => println!("获取 CodeLens 时出错: {}", e),
    // }

    // // 获取 AST
    // match client.get_ast(file_path) {
    //     Ok(ast) => println!("{}", serde_json::to_string_pretty(&ast)?),
    //     Err(e) => println!("获取 AST 时出错: {}", e),
    // }

    // // 获取补全建议
    // println!("\n=== 代码补全 ===");
    // match client.get_completions(file_path, line, column) {
    //     Ok(completions) => println!("{}", serde_json::to_string_pretty(&completions)?),
    //     Err(e) => println!("获取代码补全时出错: {}", e),
    // }

    // // 获取定义
    // println!("\n=== 定义位置 ===");
    // match client.goto_definition(file_path, line, column) {
    //     Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
    //     Err(e) => println!("获取定义位置时出错: {}", e),
    // }

    // // 获取语义标记
    // match client.get_semantic_tokens_full(file_path) {
    //     Ok(tokens) => println!("{}", serde_json::to_string_pretty(&tokens)?),
    //     Err(e) => println!("获取语义标记时出错: {}", e),
    // }

    // println!("\n=== 符号信息 ===");
    // match client.get_symbols(file_path) {
    //     Ok(symbols) => {
    //         println!("{}", serde_json::to_string_pretty(&symbols)?);
    //     }
    //     Err(e) => println!("获取符号信息时出错: {}", e),
    // }

    // // 获取定义
    // println!("\n=== 定义位置 ===");
    // match client.goto_definition(file_path, line, column) {
    //     Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
    //     Err(e) => println!("获取定义位置时出错: {}", e),
    // }
    // // 获取定义
    // println!("\n=== 定义位置 ===");
    // match client.goto_definition(file_path, line, column) {
    //     Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
    //     Err(e) => println!("获取定义位置时出错: {}", e),
    // }
    // // 获取定义
    // println!("\n=== 定义位置 ===");
    // match client.goto_definition(file_path, line, column) {
    //     Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
    //     Err(e) => println!("获取定义位置时出错: {}", e),
    // }
    // // 获取定义
    // println!("\n=== 定义位置 ===");
    // match client.goto_definition(file_path, line, column) {
    //     Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
    //     Err(e) => println!("获取定义位置时出错: {}", e),
    // }
    // // 获取定义
    // println!("\n=== 定义位置 ===");
    // match client.goto_definition(file_path, line, column) {
    //     Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
    //     Err(e) => println!("获取定义位置时出错: {}", e),
    // }

    // let _a: Value = client.reader(-1)?;
    info!("LSP 客户端示例运行完成");

    Ok(())
}
