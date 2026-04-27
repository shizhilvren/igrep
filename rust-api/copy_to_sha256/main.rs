use clap::Parser;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

const HASH_LENGTH: usize = 8;

/// A simple file copier with optional compression.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source file or directory path
    from_path: PathBuf,

    /// Destination directory path
    to_path: PathBuf,

    /// Use Zstandard compression
    #[arg(long)]
    zstd: bool,
}

fn main() {
    let args = Args::parse();

    let from_path = args.from_path.as_path();
    let to_path = args.to_path.as_path();

    // Ensure the destination directory exists
    if let Err(e) = fs::create_dir_all(to_path) {
        eprintln!("Error creating destination directory: {}", e);
        process::exit(1);
    }

    if from_path.is_file() {
        if let Ok(tar_file) = File::open(from_path) {
            let mut archive = tar::Archive::new(BufReader::new(tar_file));
            if let Err(e) = handle_tar(&mut archive, to_path, args.zstd) {
                eprintln!("Error handling tar file: {}", e);
                process::exit(1);
            }
        } else {
            eprintln!("Error opening file: {:?}", from_path);
            process::exit(1);
        }
    } else if from_path.is_dir() {
        if let Err(e) = handle_dir(from_path, to_path, args.zstd) {
            eprintln!("Error handling directory: {}", e);
            process::exit(1);
        }
    } else {
        eprintln!(
            "Source path does not exist or is not a file/directory: {:?}",
            from_path
        );
        process::exit(1);
    }
}

fn hash_file(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 128 * 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn handle_dir(from_path: &Path, to_path: &Path, use_compression: bool) -> io::Result<()> {
    for entry in WalkDir::new(from_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let metadata = fs::symlink_metadata(path)?;

        // Skip directories, symlinks, and special files
        if metadata.is_dir() || metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }

        let file_hash = hash_file(path)?;
        let filename = if use_compression {
            format!("{}.bin.zst", &file_hash[..HASH_LENGTH])
        } else {
            format!("{}.bin", &file_hash[..HASH_LENGTH])
        };
        let to_abs = to_path.join(&filename);

        if to_abs.exists() {
            println!("Exists, skipped {:?} ({:?})", to_abs, path);
        } else {
            if use_compression {
                println!("Compressing {:?} {:?}", path, to_abs);
                let src_file = File::open(path)?;
                let dst_file = File::create(&to_abs)?;
                let mut writer = BufWriter::new(dst_file);
                let mut encoder = zstd::Encoder::new(writer, 19)?;
                io::copy(&mut BufReader::new(src_file), &mut encoder)?;
                encoder.finish()?;
            } else {
                println!("cp {:?} {:?}", path, to_abs);
                fs::copy(path, &to_abs)?;
            }
        }
    }
    Ok(())
}

fn handle_tar<R: Read>(
    archive: &mut tar::Archive<R>,
    to_path: &Path,
    use_compression: bool,
) -> io::Result<()> {
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let header = entry.header();

        if header.entry_type().is_file() || header.entry_type().is_hard_link() {
            let path_str = String::from_utf8_lossy(&header.path_bytes()).to_string(); // 修复：添加 & 符号
            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let file_hash = hex::encode(hasher.finalize());

            let filename = if use_compression {
                format!("{}.bin.zst", &file_hash[..HASH_LENGTH])
            } else {
                format!("{}.bin", &file_hash[..HASH_LENGTH])
            };
            let to_abs = to_path.join(&filename);

            if to_abs.exists() {
                println!("Exists, skipped {:?} ({})", to_abs, path_str);
            } else {
                if use_compression {
                    println!("Extracted and compressing {:?} ({})", to_abs, path_str);
                    let dst_file = File::create(&to_abs)?;
                    let mut encoder = zstd::Encoder::new(dst_file, 19)?; // 修复：直接将 dst_file 传入，移除不必要的 BufWriter
                    encoder.write_all(&content)?;
                    encoder.finish()?;
                } else {
                    println!("Extracted {:?} ({})", to_abs, path_str);
                    let mut dst_file = File::create(&to_abs)?;
                    dst_file.write_all(&content)?;
                }
            }
        }
    }
    Ok(())
}
