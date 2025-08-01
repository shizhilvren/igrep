use crate::lsp::clangd_client::ClangdClient;
use anyhow::Result;
use std::{fs, thread::sleep, time::Duration};

pub fn main(
    file_path: &str,
    project_dir: &str,
    debug: bool,
    line: u32,
    column: u32,
    log_file: String,
) -> Result<()> {
    // 初始化客户端，连接到本地运行的 clangd 服务器
    println!("正在连接到 clangd 服务器...");
    let mut client = ClangdClient::new(&log_file)?;
    println!("已连接到 clangd 服务器");

    let file_content = fs::read_to_string(file_path)?;

    // 在 LSP 服务器中打开文件
    client.open_file(file_path, file_content)?;
    println!("已打开文件: {}", file_path);

    println!("获取位置 {}:{} 的信息...", line, column);

    // 获取悬停信息
    println!("\n=== 悬停信息 ===");
    match client.get_hover(file_path, line, column) {
        Ok(hover) => println!("{}", serde_json::to_string_pretty(&hover)?),
        Err(e) => println!("获取悬停信息时出错: {}", e),
    }

    // 获取 AST
    match client.get_ast(file_path) {
        Ok(ast) => println!("{}", serde_json::to_string_pretty(&ast)?),
        Err(e) => println!("获取 AST 时出错: {}", e),
    }

    // 获取补全建议
    println!("\n=== 代码补全 ===");
    match client.get_completions(file_path, line, column) {
        Ok(completions) => println!("{}", serde_json::to_string_pretty(&completions)?),
        Err(e) => println!("获取代码补全时出错: {}", e),
    }

    // 获取定义
    println!("\n=== 定义位置 ===");
    match client.goto_definition(file_path, line, column) {
        Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
        Err(e) => println!("获取定义位置时出错: {}", e),
    }

    // 获取语义标记
    match client.get_semantic_tokens_full(file_path) {
        Ok(tokens) => println!("{}", serde_json::to_string_pretty(&tokens)?),
        Err(e) => println!("获取语义标记时出错: {}", e),
    }

    println!("\n=== 符号信息 ===");
    match client.get_symbols(file_path) {
        Ok(symbols) => {
            println!("{}", serde_json::to_string_pretty(&symbols)?);
        }
        Err(e) => println!("获取符号信息时出错: {}", e),
    }

    // 获取定义
    println!("\n=== 定义位置 ===");
    match client.goto_definition(file_path, line, column) {
        Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
        Err(e) => println!("获取定义位置时出错: {}", e),
    }
        // 获取定义
    println!("\n=== 定义位置 ===");
    match client.goto_definition(file_path, line, column) {
        Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
        Err(e) => println!("获取定义位置时出错: {}", e),
    }
        // 获取定义
    println!("\n=== 定义位置 ===");
    match client.goto_definition(file_path, line, column) {
        Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
        Err(e) => println!("获取定义位置时出错: {}", e),
    }
        // 获取定义
    println!("\n=== 定义位置 ===");
    match client.goto_definition(file_path, line, column) {
        Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
        Err(e) => println!("获取定义位置时出错: {}", e),
    }
        // 获取定义
    println!("\n=== 定义位置 ===");
    match client.goto_definition(file_path, line, column) {
        Ok(definition) => println!("{}", serde_json::to_string_pretty(&definition)?),
        Err(e) => println!("获取定义位置时出错: {}", e),
    }

    sleep(Duration::from_secs(600)); // 等待一段时间以确保所有输出都能显示
    println!("\nLSP 客户端示例运行完成");

    Ok(())
}
