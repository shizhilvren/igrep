use std::{fs, io::BufRead, ops::Mul, path::PathBuf, sync::Arc};

use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::lsp::{self, builder::Builder, index::FileIndex};

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
    mut client: crate::clang::lsp_server_wraper::Client,
    semaphore: Arc<Semaphore>,
    file_index: FileIndex,
    file_path: String,
    file_lines: Vec<String>,
) -> Result<(FileIndex, lsp::data::FileSemanticTokensData)> {
    let permit = semaphore
        .acquire_owned()
        .await
        .map_err(|e| anyhow!("semaphore acquire fail: {}", e))?;
    client.did_open(&file_path, &file_lines)?;
    let semantic_tokens_response = client.semantic_tokens_full(&file_path)?;
    let semantic_tokens_response = semantic_tokens_response
        .await
        .map_err(|e| anyhow!("semantic token response recv fail: {}", e))?;
    client.did_close(&file_path)?;
    drop(permit);
    debug!("获取语义标记成功: {}", file_path);
    let semantic_tokens: lsp_types::SemanticTokens =
        serde_json::from_value(semantic_tokens_response.val)?;
    let semantic_tokens_data = lsp::data::FileSemanticTokensData::from(semantic_tokens);
    Ok((file_index, semantic_tokens_data))
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
        let semaphore = Arc::new(Semaphore::new(worker_threads.mul(2).max(1)));
        let progress_bar = ProgressBar::new(total as u64);
        progress_bar.set_message("semantic tokens");
        if let Ok(style) = ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
        ) {
            progress_bar.set_style(style.progress_chars("=> "));
        }

        file_index_data_builder
            .file_builders()
            .iter()
            .for_each(|file_builder| {
                let client = client_to_request_sender.clone();
                let semaphore = Arc::clone(&semaphore);
                let file_index = file_builder.file_index().clone();
                let file_path = file_builder
                    .file_index()
                    .path()
                    .to_string_lossy()
                    .to_string();
                let file_lines = file_builder.file_data().lines().to_vec();

                join_set.spawn(async move {
                    fetch_file_semantic_tokens(client, semaphore, file_index, file_path, file_lines)
                        .await
                });
            });

        let mut semantic_token = Vec::with_capacity(total);
        while let Some(task_result) = join_set.join_next().await {
            let task_result =
                task_result.map_err(|e| anyhow!("semantic token task join fail: {}", e))?;
            let (file_index, semantic_tokens_data) = task_result?;
            progress_bar.set_message(file_index.path().to_string_lossy().to_string());
            semantic_token.push((file_index, semantic_tokens_data));
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("semantic tokens done");
        Ok::<Vec<_>, anyhow::Error>(semantic_token)
    })?;
    info!("all semantic tokens get finish.");

    if debug {
        semantic_token
            .iter()
            .try_for_each(|(file_index, semantic_tokens)| {
                info!(
                    "文件: {:?}, 语义标记数量: {}",
                    file_index.path(),
                    semantic_tokens.tokens().len()
                );
                Ok::<(), anyhow::Error>(())
            })?;
    }

    let builder = Builder::try_from((file_index_data_builder, semantic_token))?;
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
