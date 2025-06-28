use std::{
    fs,
    io::{self, BufRead, Read, Seek},
    time::Instant,
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use igrep::{
    self,
    index_builder::NgramIndex,
    index_file::{self, FileData, FileLineData, FromToData, NgramData},
    index_regex,
};
use regex_syntax::{
    hir::{Hir, HirKind, Literal},
    parse,
};

/// Indexed grep tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index files for faster searching
    Index(IndexArgs),
    /// Search through indexed files
    Search(SearchArgs),
}

#[derive(Parser)]
struct IndexArgs {
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

#[derive(Parser)]
struct SearchArgs {
    /// Sets the config file path
    #[arg(short, long, required = true)]
    config: String,

    /// The search term to look for
    #[arg(required = true)]
    search_term: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Welcome to igrep!");
    }

    match cli.command {
        Commands::Index(args) => run_index(args, cli.verbose),
        Commands::Search(args) => run_search(args, cli.verbose),
    }
}

fn run_index(args: IndexArgs, verbose: bool) -> Result<()> {
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
        if verbose {
            print!("Indexing file {}/{}: {}", index + 1, total_files, file_name);
        }

        // 记录开始时间
        let start_time = Instant::now();

        // 索引文件
        let result = builder.index(file_name.clone(), args.ngram);

        // 计算并打印索引时间
        let duration = start_time.elapsed();

        if let Err(e) = result {
            eprintln!("Error indexing file: {} - {}", file_name, e);
        } else if verbose {
            println!("in {:.2?}", duration);
        }
    }
    builder.dump()?;

    Ok(())
}

fn run_search(args: SearchArgs, verbose: bool) -> Result<()> {
    if verbose {
        println!("Searching for '{}' in indexed files...", args.search_term);
    }

    // 现在索引搜索功能还没有实现，所以这里仅添加基础结构
    println!("Using config directory: {}", args.config);
    println!("Search term: {}", args.search_term);

    let mut idx_file = fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(format!("{}/igrep.idx", args.config))?;
    let mut idx_buf = Vec::new();
    idx_file.read_to_end(&mut idx_buf)?;
    let index_data = index_file::IndexData::from_data(idx_buf)?;
    index_data.show_info();
    let ngram = NgramIndex::new(args.search_term.as_bytes());
    let ngram_range = index_data.get_ngram_range(&ngram);
    println!("Ngram range: {:?}", ngram_range);
    if let Some(range) = ngram_range {
        let file = format!("{}/igrep.dat", args.config);
        let data = read_range(&file, range.0.start, range.0.start + range.0.len)?;
        let ngram_data = NgramData::from_data(data)?;
        println!("Ngram data: {:?}", &ngram_data);
        ngram_data
            .file_lines()
            .into_iter()
            .for_each(|file_line_index| {
                let file_range = index_data
                    .get_file_range(file_line_index.file_id())
                    .unwrap();

                let file_data = read_range(
                    &format!("{}/igrep.dat", args.config),
                    file_range.0.start,
                    file_range.0.start + file_range.0.len,
                )
                .unwrap();
                let file_data = FileData::from_data(file_data).unwrap();

                let file_line_range = file_data.lines_range(file_line_index.line_id()).unwrap();
                let file_line_data = read_range(
                    &format!("{}/igrep.dat", args.config),
                    file_line_range.0.start,
                    file_line_range.0.start + file_line_range.0.len,
                )
                .unwrap();
                let file_line_data = FileLineData::from_data(file_line_data).unwrap();

                println!(
                    "{}:{} {}",
                    file_data.name(),
                    file_line_index.line_id().line_number(),
                    file_line_data.get()
                );
            });
    }
    let engine = index_regex::Engine::new(args.search_term.as_str())?;
    let tree= engine.ngram(3);
    let simple_tree= tree.clone().simple();
    println!("Ngram tree: {:?}", &tree);
    println!("Ngram simple tree: {:?}", &simple_tree);
    let regex = parse(args.search_term.as_str())?;
    println!("Parsed regex: {:?}", regex);

    Ok(())
}

fn read_range(file: &str, start: usize, end: usize) -> Result<Vec<u8>, std::io::Error> {
    println!(
        "Reading range {}-{} size: {} from file: {}",
        start,
        end,
        end - start,
        file
    );
    let mut file = fs::File::open(file)?;
    let mut buffer = vec![0; end - start];
    file.seek(std::io::SeekFrom::Start(start as u64))?;
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
