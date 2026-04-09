use anyhow::{Result, anyhow};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, info, trace};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{fs, io::BufRead, ops::Mul, path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::lsp::{self, builder::Builder, data::HoversData, index::FileIndex};

fn init_lsp_client(
    rt: &tokio::runtime::Runtime,
    log_file: String,
    compile_commands_dir: String,
    debug: bool,
    jobs: Option<usize>,
) -> Result<crate::clang::lsp_server_wraper::Client> {
    let compile_commands_path = PathBuf::from(&compile_commands_dir).join("compile_commands.json");
    if !compile_commands_path.is_file() {
        return Err(anyhow!(
            "missing required file: {}",
            compile_commands_path.display()
        ));
    }

    let handle: tokio::task::JoinHandle<
        Result<crate::clang::lsp_server_wraper::Client, anyhow::Error>,
    > = rt.spawn(async move {
        let client_wrapper = crate::clang::lsp_server_wraper::ClangdClient::new(
            &log_file,
            compile_commands_dir,
            debug,
            jobs,
        )?;
        let client_to_request_sender = client_wrapper.warpper_loop().await?;
        let rec = client_to_request_sender.initialize()?;
        let data = rec
            .await
            .map_err(|e| anyhow!("get init response fail: {}", e))?;
        let capabilities: lsp_types::InitializeResult = serde_json::from_value(data.val)?;
        trace!("LSP server capabilities: {:?}", capabilities);
        let semanctic_tokens_server = capabilities.capabilities.semantic_tokens_provider.map_or(
            Err(anyhow!("lsp server not support semantic tokens")),
            |s| match s {
                lsp_types::SemanticTokensServerCapabilities::SemanticTokensOptions(s) => {
                    Ok(s.legend)
                }
                lsp_types::SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    s,
                ) => Ok(s.semantic_tokens_options.legend),
            },
        )?;
        client_to_request_sender.initialized()?;
        let mut client_to_request_sender = client_to_request_sender;
        client_to_request_sender.set_semantic_tokens_server(semanctic_tokens_server);
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
    progress_bar: Option<&ProgressBar>,
) -> Result<lsp_types::SemanticTokens> {
    let semantic_tokens_response = client.semantic_tokens_full(file_path)?;
    let semantic_tokens_response = semantic_tokens_response
        .await
        .map_err(|e| anyhow!("semantic token response recv fail: {}", e))?;
    debug!("获取语义标记成功: {}", file_path);
    let semantic_tokens: lsp_types::SemanticTokens =
        serde_json::from_value(semantic_tokens_response.val)?;
    if let Some(progress_bar) = progress_bar {
        progress_bar.inc(1);
    }
    Ok(semantic_tokens)
}

pub async fn handle_semantics(
    client: &mut crate::clang::lsp_server_wraper::Client,
    file_path: &str,
    tokens: &lsp_types::SemanticTokens,
    hover_progress_bar: Option<&ProgressBar>,
) -> Result<HoversData> {
    let keyword_token_type_index = client
        .get_semantic_tokens_server()
        .and_then(|legend| {
            legend.token_types.iter().position(|token_type| {
                token_type.as_str() == lsp_types::SemanticTokenType::KEYWORD.as_str()
            })
        })
        .map(|index| index as u32);
    let operator_token_type_index = client
        .get_semantic_tokens_server()
        .and_then(|legend| {
            legend.token_types.iter().position(|token_type| {
                token_type.as_str() == lsp_types::SemanticTokenType::OPERATOR.as_str()
            })
        })
        .map(|index| index as u32);
    let comment_token_type_index = client
        .get_semantic_tokens_server()
        .and_then(|legend| {
            legend.token_types.iter().position(|token_type| {
                token_type.as_str() == lsp_types::SemanticTokenType::COMMENT.as_str()
            })
        })
        .map(|index| index as u32);

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
            Some((*row, *col, token.token_type))
        })
        .filter(|(row, col, token_type)| {
            let should_hover = keyword_token_type_index != Some(*token_type)
                && operator_token_type_index != Some(*token_type)
                && comment_token_type_index != Some(*token_type);
            if !should_hover {
                trace!(
                    "跳过 keyword/operator/comment 的悬停请求: {}:{}:{}",
                    file_path, row, col
                );
            }
            should_hover
        })
        .map(|(row, col, token_type)| (row, col, token_type))
        .collect::<Vec<_>>();

    if let Some(progress_bar) = hover_progress_bar {
        let base_len = progress_bar.length().unwrap_or(0);
        progress_bar.set_length(base_len + ans.len() as u64);
        progress_bar.set_message(format!("hover {}", file_path));
    }

    let mut hovers = Vec::new();
    for (row, col, token_type) in ans {
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
                let token_type_name = client
                    .get_semantic_tokens_server()
                    .and_then(|legend| legend.token_types.get(token_type as usize))
                    .map(|t| t.as_str())
                    .unwrap_or("<unknown>");
                trace!(
                    "Failed to parse hover response: {}, token_type: {} ({})",
                    e, token_type, token_type_name
                );
            }
        }
        if let Some(progress_bar) = hover_progress_bar {
            progress_bar.inc(1);
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
    jobs: Option<usize>,
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

    let file_content = fs::read(file_list)?;
    let files_list: Vec<String> = std::io::BufReader::new(&file_content[..])
        .lines()
        .filter_map(Result::ok)
        .filter(|file| !file.is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    info!("build file index");
    let mut file_index_builder = lsp::builder::FileIndexBuilder::from(());
    files_list.into_iter().try_for_each(|file_name| {
        let file_index = FileIndex::from(file_name);
        file_index_builder.insert(file_index)?;
        Ok::<(), anyhow::Error>(())
    })?;
    info!("file index build done, start read file content");
    let file_index_data_builder = lsp::builder::FileIndexDataBuilder::try_from(file_index_builder)?;
    info!("file content read done, start init lsp client");

    let client_to_request_sender =
        init_lsp_client(&rt, log_file, compile_commands_dir, debug, jobs)?;
    let (client_to_request_sender, file_index_data_builder) =
        wait_index_done(&rt, client_to_request_sender, file_index_data_builder)?;
    info!("index done");

    let data_tokens = rt.block_on(async {
        let mut join_set = JoinSet::new();
        let total = file_index_data_builder.file_builders().len();
        let semaphore = Arc::new(Semaphore::new(worker_threads.mul(1).max(1).min(9999)));
        let multi_progress = Arc::new(MultiProgress::new());
        let progress_bar = multi_progress.add(ProgressBar::new(total as u64));
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
                let multi_progress = Arc::clone(&multi_progress);

                join_set.spawn(async move {
                    let result = async {
                        let _open_span_permit = semaphore
                            .acquire_owned()
                            .await
                            .map_err(|e| anyhow!("semaphore acquire fail: {}", e))?;

                        let hover_progress_bar = multi_progress.add(ProgressBar::new_spinner());
                        if let Ok(style) = ProgressStyle::with_template(
                            "  [{elapsed_precise}] [{bar:30.green/black}] {pos}/{len} {msg}",
                        ) {
                            hover_progress_bar.set_style(style.progress_chars("=> "));
                        }
                        hover_progress_bar.enable_steady_tick(Duration::from_millis(700));
                        hover_progress_bar.set_length(1);
                        hover_progress_bar.set_position(0);
                        hover_progress_bar.set_message(format!("semantic {}", file_path));

                        trace!("start to get semantic tokens: {}", file_path);
                        client.did_open(&file_path, &file_lines)?;

                        let semantic_and_hovers_result = async {
                            let semantic_tokens = fetch_file_semantic_tokens(
                                &mut client,
                                &file_path,
                                Some(&hover_progress_bar),
                            )
                            .await?;
                            trace!("semantic tokens get finish: {}", file_path);

                            let hovers = handle_semantics(
                                &mut client,
                                &hover_file_path,
                                &semantic_tokens,
                                Some(&hover_progress_bar),
                            )
                            .await?;
                            Ok::<_, anyhow::Error>((semantic_tokens, hovers))
                        }
                        .await;

                        hover_progress_bar.finish_and_clear();
                        let (semantic_tokens, hovers) = semantic_and_hovers_result?;

                        trace!("semantic tokens get finish: {}", file_path);

                        client.did_close(&file_path)?;
                        trace!("semantic tokens and hovers get finish: {}", file_path);
                        let semantic_tokens_data =
                            lsp::data::FileSemanticTokensData::from(semantic_tokens);

                        Ok::<_, anyhow::Error>((file_index, semantic_tokens_data, hovers))
                    }
                    .await;
                    result
                });
            });

        let mut data_tokens = Vec::with_capacity(total);
        while let Some(task_result) = join_set.join_next().await {
            let task_result: Result<_, anyhow::Error> =
                task_result.map_err(|e| anyhow!("semantic token task join fail: {}", e))?;
            let (file_index, semantic_tokens_data, hovers) = task_result?;
            progress_bar.set_message(file_index.path().to_string_lossy().to_string());
            data_tokens.push((file_index, semantic_tokens_data, hovers));
            progress_bar.inc(1);
        }
        progress_bar.finish_with_message("semantic tokens done");
        Ok::<Vec<_>, anyhow::Error>(data_tokens)
    })?;
    info!("all semantic tokens get finish.");

    if debug {
        data_tokens
            .iter()
            .try_for_each(|(file_index, semantic_tokens, hovers)| {
                info!(
                    "文件: {:?}, 语义标记数量: {} , 悬停信息数量: {} 总字节数: {}",
                    file_index.path(),
                    semantic_tokens.tokens().len(),
                    hovers.hovers().len(),
                    hovers
                        .hovers()
                        .iter()
                        .map(|t| t.hover().len() as u64)
                        .sum::<u64>()
                );
                Ok::<(), anyhow::Error>(())
            })?;
    }

    let builder = Builder::try_from((file_index_data_builder, data_tokens))?;
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
