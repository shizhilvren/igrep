use std::{fs, io::BufRead, time::Instant};

use anyhow::Result;
use clap::Parser;
use igrep;

/// Indexed grep tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Sets the file to be indexed
    #[arg(short, long, required = true)]
    file_list: String,

    /// Sets the config file path
    #[arg(short, long, default_value = "test")]
    config: String,

    /// Sets the size of n-gram
    #[arg(short, long, default_value_t = 3)]
    ngram: usize,
}

fn main() -> Result<()> {
    println!("Welcome to igrep!");

    // 解析命令行参数
    let args = Args::parse();

    // 创建索引构建器
    let mut builder = igrep::index_builder::IndexBuilder::new(args.config)?;

    // 读取文件列表
    let file_content = fs::read(args.file_list.clone())?;
    let file_lines: Vec<String> = std::io::BufReader::new(&file_content[..])
        .lines()
        .filter_map(Result::ok)
        .filter(|file| !file.is_empty())
        .map(|line| line.trim().to_string())
        .collect();

    // 获取总文件数
    let total_files = file_lines.len();
    println!("Total files to index: {}", total_files);

    // 处理每个文件
    for (index, file_name) in file_lines.iter().enumerate() {
        // 显示进度
        print!("Indexing file {}/{}: {}", index + 1, total_files, file_name);

        // 记录开始时间
        let start_time = Instant::now();

        // 索引文件
        let result = builder.index(file_name.clone(), args.ngram);

        // 计算并打印索引时间
        let duration = start_time.elapsed();

        if let Err(e) = result {
            eprintln!("Error indexing file: {} - {}", file_name, e);
        } else {
            println!("in {:.2?}", duration);
        }
    }

    Ok(())
}
