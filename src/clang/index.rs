use anyhow::{Result, anyhow};
use clang::{Clang, Entity, Index};
use std::path::Path;

/// 处理单个文件
pub fn main(file: &str, dir: &str) -> Result<()> {
    let index = clang_sys::clang_createIndex(0, 0);
    let compilation_database = clang::CompilationDatabase::from_directory(dir).unwrap();
    // Acquire an instance of `Clang`
    let clang = Clang::new().unwrap();

    // Create a new `Index`
    let index = Index::new(&clang, false, false);

    let args = compilation_database.get_compile_commands(file).unwrap();
    let args = args.get_commands();
    let args = args.first().unwrap();
    println!("dir: {:?}", args.get_directory());
    println!("file: {:?}", args.get_filename());
    let args = args.get_arguments().iter().cloned().collect::<Vec<_>>();
    println!("args: {:?}", &args);
    let args = vec![
        "/remote/vgfdn2_hw_loaner/tools/depot/qsc/QSCW/GCC/bin/g++",
        "--driver-mode=g++",
        "-I/depot/ipp-7.0.6.273",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug/vcs-src",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug",
        "-I/depot/ipp-7.0.6.273",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug/vcs-src",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug",
        "-B/remote/vgfdn2_hw_loaner/tools/global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/llvm-15.0.2/bin",
        "-I/remote/vg_thirdparty/vg_foundation/QSCX_270325/snps_boost_1_85_0",
        "-I/remote/vg_thirdparty/vg_foundation/QSCX_270325/snps_boost_1_85_0",
        "-I/remote/vg_thirdparty/vg_foundation/QSCX_270325",
        "-I/global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/lld_linker_thirdparty/Ctemplate/linux64/include",
        "-I/global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/zstd-1.4.0/include",
        "-I/remote/vgfdn2_hw_loaner/tools//global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/zlib-1.2.12_UNIFIED/linux64/include",
        "-I/global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/zstd-1.4.0/include",
        "-I/remote/vgfdn2_hw_loaner/tools//global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/zlib-1.2.12_UNIFIED/linux64/include",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug/verdi-src/bt/kdb",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug/verdi-src/bt/kdb/inc",
        "-I/linux64/include",
        "-I/remote/vg_thirdparty/vg_foundation/QSCX_270325/common/include",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug/vgcommon/plato_test/plato_code/plato-src/include",
        "-I/remote/vcs_source02/lisimon/code/td/td1_debug/vgcommon/plato_test/plato_code/plato-src/loader",
        "-I/remote/vg_thirdparty/vg_foundation/QSCX_270325/snps_boost_1_85_0",
        "-I/global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/Mustache/v4.0",
        "-Werror",
        "-Wall",
        "-Wextra",
        "-Wpointer-arith",
        "-Wformat=2",
        "-Wmissing-braces",
        "-Wno-format-nonliteral",
        "-Wno-missing-field-initializers",
        "-Wno-unused-but-set-parameter",
        "-Wno-unused-but-set-variable",
        "-Wno-unused-local-typedefs",
        "-Wno-unused-parameter",
        "-fdiagnostics-show-option",
        "-fno-dollars-in-identifiers",
        "-Wno-ignored-qualifiers",
        "-Wno-error=deprecated-declarations",
        "-Wno-error=cpp",
        "-DFSDB_FOR_VCS",
        "-I",
        "/remote/vg_thirdparty/vg_foundation/QSCX_270325/common/spl/snps/include/spl",
        "-I",
        "/remote/vg_thirdparty/vg_foundation/QSCX_270325/common/spl/snps/include",
        "-DENGINEER",
        "-DVCS_TARGET_ARCH=\"linux64\"",
        "-DINST64_ENABLE",
        "-DSynopsys_Boost_Full_Set",
        "-B",
        "/remote/vgfdn2_hw_loaner/tools/global/artifacts/vg_thirdparty/TP_092024/fs/src/interfaces/llvm-15.0.2/bin",
        "-Xassembler",
        "-mrelax-relocations=no",
        "-DVCS_SV2CPP_ENABLED",
        "-Wno-register",
        "-Dlinux",
        // "-Wno-literal-suffix",
        "-DUSER_FOREIGN_OZ_WRITER",
        "-m64",
        "-msse2",
        "-DVCS64_FLAG",
        "-fPIC",
        "-DSTATIC_LIBRARY",
        "-D__NO_STRING_INLINES",
        "-DVCSCPU_X86_64",
        "-mstackrealign",
        "-DFGP_ENABLE_TLS",
        "-DPEBLK_THREAD",
        "-DINST64_ENABLE",
        "-fuse-ld=lld",
        "-fcommon",
        "-DLINUX",
        "-DSynopsys_linux",
        "-Wno-error=array-bounds",
        "-m64",
        "-msse2",
        "-DVCS64_FLAG",
        "-fPIC",
        "-DSTATIC_LIBRARY",
        "-D__NO_STRING_INLINES",
        "-DVCSCPU_X86_64",
        "-mstackrealign",
        "-DFGP_ENABLE_TLS",
        "-DPEBLK_THREAD",
        "-DINST64_ENABLE",
        "-fuse-ld=lld",
        "-fcommon",
        "-DLINUX",
        "-DSynopsys_linux",
        "-Wno-error=array-bounds",
        "-o",
        "/remote/vcs_source02/lisimon/code/td/td1_debug/OBJ/vcs-src/lp-src/verilog/obj-linux64/lp_utils_vir.o",
        "-O3",
        "-msse2",
        "-fno-strict-aliasing",
        "-DNO_DEBUG",
        "-D__NO_STRING_INLINES",
        "-fno-omit-frame-pointer",
        "-c",
        "-DVCS64_FLAG",
        // "/remote/vcs_source02/lisimon/code/td/td1_debug/vcs-src/lp-src/verilog/lp_utils_vir.cc",
        // "lp_utils_vir.cc",
    ];
    // Parse a source file into a translation unit
    let mut parser = index.parser(file);
    let parser = parser.arguments(args.as_slice());
    let parser = parser
        // .cache_completion_results(true)
        .detailed_preprocessing_record(true);
    let tu = parser.parse()?;
    dfs(&tu.get_entity());
    Ok(())
}

// /// 处理多个文件
// pub fn process_multiple_files(files: &[&str], dir: &str) -> Result<()> {
//     let compilation_database = clang::CompilationDatabase::from_directory(dir)?;
//     let clang = Clang::new()?;
//     let index = Index::new(&clang, false, false);

//     // 存储所有解析的翻译单元
//     let mut translation_units = Vec::new();

//     for &file in files {
//         println!("Processing file: {}", file);

//         // 尝试获取编译命令
//         let commands = match compilation_database.get_compile_commands(file) {
//             Some(cmds) => cmds,
//             None => {
//                 println!(
//                     "Warning: No compilation commands found for {}, skipping",
//                     file
//                 );
//                 continue;
//             }
//         };

//         let command_list = commands.get_commands();
//         if command_list.is_empty() {
//             println!("Warning: Empty compilation commands for {}, skipping", file);
//             continue;
//         }

//         let command = command_list.first().unwrap();
//         println!("dir: {:?}", command.get_directory());
//         println!("file: {:?}", command.get_filename());

//         let args: Vec<_> = command.get_arguments().iter().cloned().collect();

//         // 解析文件
//         let mut parser = index.parser(file);
//         let parser = parser.arguments(&args);
//         let parser = parser.cache_completion_results(true);

//         match parser.parse() {
//             Ok(tu) => {
//                 println!("Successfully parsed: {}", file);
//                 translation_units.push(tu);
//             }
//             Err(e) => {
//                 println!("Failed to parse {}: {:?}", file, e);
//             }
//         }
//     }

//     // 处理所有翻译单元
//     for (i, tu) in translation_units.iter().enumerate() {
//         println!("=== Translation Unit {} ===", i);
//         dfs(&tu.get_entity());
//     }

//     Ok(())
// }

// /// 处理一个目录中的所有C/C++文件
// pub fn process_directory(dir_path: &str, compilation_db_dir: &str) -> Result<()> {
//     let extensions = ["c", "cpp", "cc", "cxx", "h", "hpp", "hxx"];
//     let mut files = Vec::new();

//     // 遍历目录收集所有C/C++文件
//     collect_files(Path::new(dir_path), &extensions, &mut files)?;

//     // 转换为字符串引用的切片
//     let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();

//     // 处理所有文件
//     process_multiple_files(&file_refs, compilation_db_dir)?;

//     Ok(())
// }

// // 辅助函数：收集指定目录中的所有匹配扩展名的文件
// fn collect_files(dir: &Path, extensions: &[&str], files: &mut Vec<String>) -> Result<()> {
//     if !dir.is_dir() {
//         return Err(anyhow!("{:?} is not a directory", dir));
//     }

//     for entry in std::fs::read_dir(dir)? {
//         let entry = entry?;
//         let path = entry.path();

//         if path.is_dir() {
//             // 递归处理子目录
//             collect_files(&path, extensions, files)?;
//         } else if let Some(ext) = path.extension() {
//             if let Some(ext_str) = ext.to_str() {
//                 if extensions.contains(&ext_str) {
//                     if let Some(path_str) = path.to_str() {
//                         files.push(path_str.to_owned());
//                     }
//                 }
//             }
//         }
//     }

//     Ok(())
// }

fn dfs(entity: &Entity) {
    println!("{:?}", &entity);
    println!("\t definition {:?}", entity.get_definition());
    println!("\t reference {:?}", entity.get_reference());
    println!(
        "\t overridden methods {:?}",
        entity.get_overridden_methods()
    );
    println!("\t mangled name {:?}", entity.get_mangled_name());
    println!("\t usr {:?}", entity.get_usr());
    for child in entity.get_children() {
        dfs(&child);
    }
}
