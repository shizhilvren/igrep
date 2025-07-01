mod builder;
mod config;
mod data;
mod encode;
mod index;
mod range;

use std::{
    fs::{self, File},
    io::{BufRead, Read, Seek},
    time::Instant,
};

use crate::{
    builder::{AbsPath, Buuilder, FileContent, FileIndexBuilder},
    index::{FileIndex, FileLineIndex, LineIndex, NgramIndex},
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use regex::Regex;
use regex_syntax::{
    ast::print,
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
        .iter()
        .filter_map(|file_name| {
            let path = AbsPath::new((*file_name).clone());
            match path {
                Ok(path) => fid_builder.insert(&path).map_or_else(
                    |e| {
                        println!("{} {:?} ", file_name, e);
                        None
                    },
                    |_| Some(path),
                ),
                Err(e) => {
                    println!("{} {:?} ", file_name, e);
                    None
                }
            }
        })
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
    let merges = file_lines
        .into_iter()
        .enumerate()
        .filter_map(|(id, file_content)| {
            let file_name = file_content.get_name().path().to_string();

            // 显示进度
            if verbose {
                print!("Indexing file {}/{}: {}", id + 1, total_files, &file_name);
            }

            // 记录开始时间
            let start_time = Instant::now();
            let ans = Buuilder::index(&fid_builder, file_content, args.ngram);
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
    let mut encode_data = Buuilder::merge(merges);
    encode_data.dump();

    Ok(())
}

fn run_search(args: SearchArgs, verbose: bool) -> Result<()> {
    // if verbose {
    //     println!("Searching for '{}' in indexed files...", args.search_term);
    // }

    // // 现在索引搜索功能还没有实现，所以这里仅添加基础结构
    // println!("Using config directory: {}", args.config);
    // println!("Search term: {}", args.search_term);

    // let mut idx_file = fs::OpenOptions::new()
    //     .read(true)
    //     .write(false)
    //     .open(format!("{}/igrep.idx", args.config))?;
    // let mut idx_buf = Vec::new();
    // idx_file.read_to_end(&mut idx_buf)?;
    // let index_data = index_file::IndexData::from_data(idx_buf)?;
    // index_data.show_info();

    // let engine = index_regex::Engine::new(args.search_term.as_str())?;
    // let tree = engine.ngram(3);
    // let simple_tree = tree.is_all();
    // println!("Ngram tree: {:?}", &tree);
    // println!("Ngram is all: {:?}", &simple_tree);
    // let ngrams = tree.ngrams();
    // println!("Ngrams need get: {:?}", &ngrams);
    // for ngram in &ngrams {
    //     let ngram_data = get_ngram_data(
    //         format!("{}/igrep.dat", args.config).as_str(),
    //         &index_data,
    //         &ngram,
    //     );
    //     println!("{:?} -> {}", &ngram, ngram_data.file_lines().len());
    // }

    // let index = ngrams
    //     .iter()
    //     .map(|ngram| {
    //         let ngram_data = get_ngram_data(
    //             format!("{}/igrep.dat", args.config).as_str(),
    //             &index_data,
    //             &ngram,
    //         );
    //         (ngram.clone(), ngram_data)
    //     })
    //     .collect::<HashMap<_, _>>();
    // let ref_index = index
    //     .iter()
    //     .map(|(ngram, data)| (ngram.clone(), data))
    //     .collect::<HashMap<_, _>>();
    // let result = tree.get_file_lines(&ref_index);
    // // println!("file lines {:?}", &result);
    // match result {
    //     index_regex::NgramTreeResult::ALL => println!("chat not longer then index"),
    //     index_regex::NgramTreeResult::Set(sub) => {
    //         println!("Found {} file lines matching the search term", sub.len());
    //         let re = Regex::new(args.search_term.as_str()).unwrap();
    //         sub.into_iter()
    //             .map(|e| {
    //                 let d = get_file_line_data(
    //                     format!("{}/igrep.dat", args.config).as_str(),
    //                     &index_data,
    //                     &e,
    //                 );
    //                 (e, d)
    //             })
    //             .filter(|(i, d)| re.is_match(d.0.get()))
    //             .for_each(|(i, d)| {
    //                 println!("{}:{:?}{}", d.1, i.line_id().line_number(), d.0.get());
    //             });
    //     }
    // }

    // let regex = parse(args.search_term.as_str())?;
    // println!("Parsed regex: {:?}", regex);

    Ok(())
}

// fn read_range(file: &str, start: Offset, end: Offset) -> Result<Vec<u8>, std::io::Error> {
//     // println!(
//     //     "Reading range {}-{} size: {} from file: {}",
//     //     start,
//     //     end,
//     //     end - start,
//     //     file
//     // );
//     let mut file = fs::File::open(file)?;
//     let mut buffer = vec![0; (end - start) as usize];
//     file.seek(std::io::SeekFrom::Start(start as u64))?;
//     file.read_exact(&mut buffer)?;
//     Ok(buffer)
// }

// fn get_ngram_data(file: &str, index_data: &IndexData, ngram_index: &NgramIndex) -> NgramData {
//     index_data
//         .get_ngram_range(ngram_index)
//         .and_then(|range| {
//             println!("get ngram {:?} range {:?}", &ngram_index, &range);
//             read_range(file, range.0.start, range.0.start + range.0.len as u64)
//                 .and_then(|data| NgramData::from_data(data))
//                 .map_or(None, |d| Some(d))
//         })
//         .unwrap_or(NgramData::new())
// }

// fn get_file_line_data(
//     file: &str,
//     index_data: &IndexData,
//     file_line_index: &FileLineIndex,
// ) -> (FileLineData, String) {
//     let file_range = index_data
//         .get_file_range(file_line_index.file_id())
//         .unwrap();

//     let file_data = read_range(
//         file,
//         file_range.0.start,
//         file_range.0.start + file_range.0.len as u64,
//     )
//     .unwrap();
//     let file_data = FileData::from_data(file_data).unwrap();

//     let file_line_range = file_data.lines_range(file_line_index.line_id()).unwrap();
//     let file_line_data = read_range(
//         file,
//         file_line_range.0.start,
//         file_line_range.0.start + file_line_range.0.len as u64,
//     )
//     .unwrap();
//     let file_line_data = FileLineData::from_data(file_line_data).unwrap();
//     (file_line_data, file_data.name().clone())
// }
