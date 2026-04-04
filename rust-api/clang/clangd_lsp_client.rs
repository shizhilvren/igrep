use crate::lsp;
use crate::lsp::builder::Builder;
use crate::lsp::clangd_client::ClangdClient;
use crate::lsp::data::FileSemanticTokensData;
use crate::lsp::index::FileIndex;
use anyhow::{Result, anyhow};
use log::{error, info};
use serde_json::Value;
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;

pub fn main(
    file_list: &str,
    debug: bool,
    log_file: String,
    compile_commands_dir: String,
    config: &str,
) -> Result<()> {
    let (lsp_request_tx, lsp_request_rx) = tokio::sync::mpsc::unbounded_channel::<u32>();
    let (lsp_response_tx, lsp_response_rx) = tokio::sync::mpsc::unbounded_channel::<u32>();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2) // 设置工作线程数为 4
        .thread_name("lsp-server-wrapper") // 设置线程名称
        .enable_all() // 启用 IO 和 Time 驱动
        .build()
        .unwrap();
    rt.spawn(async move {
        let client_wrapper = crate::clang::lsp_server_wraper::ClangdClient::new(
            &log_file,
            compile_commands_dir.to_string(),
            debug,
        )
        .unwrap();
        client_wrapper.warpper_loop(lsp_request_rx).await.unwrap();
    });

    println!("Wait for it...");
    std::thread::sleep(std::time::Duration::from_secs(2)); // Sleep for 2 seconds
    println!("Done!");

    // // 初始化客户端，连接到本地运行的 clangd 服务器
    // println!("正在连接到 clangd 服务器...");
    // let mut client = ClangdClient::new(&log_file, compile_commands_dir, debug)?;
    // println!("已连接到 clangd 服务器");

    // let file_content = fs::read(file_list)?;
    // let files_list: Vec<String> = std::io::BufReader::new(&file_content[..])
    //     .lines()
    //     .filter_map(Result::ok)
    //     .filter(|file| !file.is_empty())
    //     .map(|line| line.trim().to_string())
    //     .collect();

    // let first_file = files_list.first().ok_or_else(|| anyhow!("文件列表为空"))?;

    // // 在 LSP 服务器中打开文件
    // client.open_file(first_file)?;
    // info!("已打开文件: {}", first_file);

    // client.did_close(first_file)?;
    // info!("已关闭文件: {}", first_file);

    // let _: Value = client.reader(-1)?;
    // info!("LSP index finish");

    // let mut file_index_builder = lsp::builder::FileIndexBuilder::from(());
    // files_list.into_iter().try_for_each(|file_name| {
    //     let file_index = FileIndex::from(file_name);
    //     file_index_builder.insert(file_index)?;
    //     Ok::<(), anyhow::Error>(())
    // })?;
    // let file_index_data_builder = lsp::builder::FileIndexDataBuilder::try_from(file_index_builder)?;

    // let semantic_token = file_index_data_builder
    //     .file_builders()
    //     .iter()
    //     .map(|file_builder| {
    //         let file_path = file_builder
    //             .file_index()
    //             .path()
    //             .to_string_lossy()
    //             .to_string();
    //         let file_data = file_builder.file_data();
    //         client.open_file_with_data(&file_path, file_data)?;
    //         let tokens = client.get_semantic_tokens_full(file_path.as_str())?;
    //         client.did_close(&file_path)?;
    //         info!("获取语义标记成功: {}", file_path);
    //         let semantic_tokens: lsp_types::SemanticTokens = serde_json::from_value(tokens)?;
    //         let semantic_toklens_data = FileSemanticTokensData::from(semantic_tokens);
    //         Ok((file_builder.file_index().clone(), semantic_toklens_data))
    //     })
    //     .collect::<Result<Vec<_>>>()?;
    // info!("all semantic tokens get finish.");
    // // semantic_token
    // //     .into_iter()
    // //     .try_for_each(|(file_index, semantic_tokens)| {
    // //         info!(
    // //             "文件: {:?}, 语义标记数量: {}",
    // //             file_index.path(),
    // //             semantic_tokens.tokens().len()
    // //         );
    // //         Ok::<(), anyhow::Error>(())
    // //     })?;

    // let builder = Builder::try_from((file_index_data_builder, semantic_token))?;
    // builder.dump(PathBuf::from(config).as_path())?;

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
