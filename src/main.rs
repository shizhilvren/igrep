mod builder;
mod config;
mod data;
mod encode;
mod index;
mod range;
mod search;

use crate::search::{Engine, FileDataMatchRange, NgreamIndexData};
use crate::{
    builder::{AbsPath, Builder, FileContent, FileIndexBuilder},
    range::Offset,
};

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use colored::Colorize;
use rayon::prelude::*;

use std::fs;
use std::{
    io::{BufRead, Read, Seek},
    path::Path,
    time::Instant,
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
    ngram: u8,
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
    let mut fid_builder = FileIndexBuilder::new();

    let file_lines = file_lines
        .into_par_iter()
        .filter_map(|file_name| {
            let path = AbsPath::new(file_name.clone());
            match path {
                Ok(path) => Some(path),
                Err(e) => {
                    println!("{} {:?} ", file_name, e);
                    None
                }
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|path| {
            fid_builder.insert(&path).map_or_else(
                |e| {
                    println!("{} {:?} ", path.path(), e);
                    None
                },
                |_| Some(path.clone()),
            )
        })
        .collect::<Vec<_>>();
    let file_lines = file_lines
        .into_par_iter()
        .filter_map(|file| {
            let path = file.path().to_string();
            fs::read_to_string(&path).map_or_else(
                |e| {
                    println!("{:?} {:?} ", &path, e);
                    None
                },
                |lines| {
                    Some(FileContent::new(
                        file,
                        lines
                            .lines()
                            .map(String::from) // make each slice into a string
                            .collect(),
                    ))
                },
            )
        })
        .collect::<Vec<_>>();

    let total_files = file_lines.len();
    println!("all file read done");
    let fid_builder = fid_builder.make_final();
    let builder = Builder::new(args.ngram)?;
    let merges = file_lines
        .into_par_iter()
        .enumerate()
        .filter_map(|(id, file_content)| {
            let file_name = file_content.get_name().path().to_string();
            // 显示进度
            if verbose {
                print!("Indexing file {}/{}: {}", id + 1, total_files, &file_name);
            }
            // 记录开始时间
            let start_time = Instant::now();
            let ans = builder.index(&fid_builder, file_content);
            // 计算并打印索引时间
            let duration = start_time.elapsed();

            if ans.is_none() {
                eprintln!("Error indexing file: {} ", file_name);
            } else if verbose {
                println!("in {:.2?}", duration);
            }
            ans
        })
        .collect::<Vec<_>>();
    println!("Indexing completed, merging data...");
    let mut encode_data = builder.merge(merges);
    println!("Merging completed, dumping data...");
    encode_data.dump(Path::new(&args.config))?;
    println!("Indexing and merging completed successfully.");

    Ok(())
}

fn run_search(args: SearchArgs, verbose: bool) -> Result<()> {
    println!("Using config directory: {}", args.config);
    println!("Search term: {}", args.search_term);

    let idx_file_path = format!("{}/igrep.idx", args.config);
    let dat_file_path = format!("{}/igrep.dat", args.config);
    let mut idx_file = fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(idx_file_path)?;
    let mut idx_buf = Vec::new();
    idx_file.read_to_end(&mut idx_buf)?;
    let engine = Engine::new(idx_buf).map_err(|e| anyhow!(e))?;
    if verbose {
        println!("Index data loaded successfully.");
        engine.show_info();
    }
    let ngram_tree = engine
        .regex(args.search_term.as_str())
        .map_err(|e| anyhow!(e))?;
    let ngrams = engine.ngram_ranges(&ngram_tree);
    let ngram_index_datas = ngrams
        .into_par_iter()
        .map(|idx_range| {
            let start = idx_range.range.0.start;
            let end = idx_range.range.0.start + idx_range.range.0.len as u64;
            read_range(dat_file_path.as_str(), start, end)
                .and_then(|r| Ok(NgreamIndexData::new(idx_range.index(), r)))
                .map_err(|e| anyhow!("Failed to read range: {}", e))
        })
        .collect::<Result<Vec<_>>>()?;
    let ngram_tree_result_struct = engine
        .get_search_result(&ngram_tree, ngram_index_datas)
        .map_err(|e| anyhow!(e))?;

    if ngram_tree_result_struct.is_all() {
        println!("search len smail then 3");
    } else {
        ngram_tree_result_struct
            .file_lines()
            .map_err(|e| anyhow!(e))?
            .into_iter()
            .map(|file_lines| {
                let r = engine.file_range(&file_lines.file).unwrap();
                let start = r.0.start;
                let end = r.0.start + r.0.len as u64;
                let data = read_range(dat_file_path.as_str(), start, end).unwrap();
                let file_data = engine.build_file_data(data).unwrap();
                let file_name = file_data.name();
                let file_lines = file_lines
                    .lines()
                    .into_iter()
                    .filter_map(|line| {
                        let r = file_data.lines_range(&line).unwrap();
                        let start = r.0.start;
                        let end = r.0.start + r.0.len as u64;
                        let data = read_range(dat_file_path.as_str(), start, end).unwrap();
                        let line_data = engine.build_file_line_data(data).unwrap();
                        let match_ranges =
                            engine.file_data_match(&line_data, &ngram_tree_result_struct);
                        if match_ranges.is_empty() {
                            return None;
                        } else {
                            Some((line.line_number(), line_data.get(), match_ranges))
                        }
                    })
                    .collect::<Vec<_>>();
                (file_name, file_lines)
            })
            .filter_map(|(name, lines)| {
                if lines.is_empty() {
                    None
                } else {
                    Some((name, lines))
                }
            })
            .for_each(|(name, lines)| {
                println!("{}", name.purple());
                for (line_number, line_content, match_ranges) in lines {
                    for FileDataMatchRange { start, end } in match_ranges {
                        // Split the string to highlight the matched part
                        let before = &line_content[..start as usize];
                        let matched = &line_content[start as usize..end as usize].red();
                        let after = &line_content[end as usize..];

                        println!(
                            "{}: {}{}{}",
                            line_number.to_string().green(),
                            before,
                            matched,
                            after
                        );
                    }
                }
            });
    }

    println!("Parsed regex: {:?}", args.search_term);

    Ok(())
}

fn read_range(file: &str, start: Offset, end: Offset) -> Result<Vec<u8>, std::io::Error> {
    // println!(
    //     "Reading range {}-{} size: {} from file: {}",
    //     start,
    //     end,
    //     end - start,
    //     file
    // );
    let mut file = fs::File::open(file)?;
    let mut buffer = vec![0; (end - start) as usize];
    file.seek(std::io::SeekFrom::Start(start as u64))?;
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}
