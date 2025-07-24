use crate::clang::json::{FileJson, SymbolsJson};
use anyhow::{Result, anyhow};
use clang::{Clang, Entity, Index, Usr, source::SourceLocation};
use regex_syntax::ast::print;
use std::io::Write;
use std::{
    collections::{HashMap, HashSet},
    fs,
    hash::Hash,
    path::Path,
};

const FILE_JSON_PREFIX: &str = "web/index/file/";
const SYMBOL_JSON_PREFIX: &str = "web/index/symbol/";

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd)]
pub(super) struct FileLocation {
    file: String,
    loc: OneFileLocation,
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd)]
pub(super) struct OneFileLocation {
    line: u32,
    column: u32,
    offset: u32,
    len: u32,
}

#[derive(Debug)]
pub(super) struct FunctionResult {
    declarations: HashSet<FileLocation>,
    // function body
    definitions: HashSet<FileLocation>,
    calls: HashSet<FileLocation>,
}

#[derive(Debug)]
pub(super) struct IndexResult {
    functions: HashMap<Usr, FunctionResult>,
}

impl OneFileLocation {
    pub fn offset(&self) -> u32 {
        self.offset
    }
    pub fn len(&self) -> u32 {
        self.len
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn column(&self) -> u32 {
        self.column
    }
}
impl FileLocation {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn loc(&self) -> &OneFileLocation {
        &self.loc
    }
}
impl Ord for FileLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.file
            .cmp(&other.file)
            .then_with(|| self.loc.cmp(&other.loc))
    }
}

impl Ord for OneFileLocation {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.column.cmp(&other.column))
            .then_with(|| self.offset.cmp(&other.offset))
            .then_with(|| self.len.cmp(&other.len))
    }
}

impl IndexResult {
    pub fn new() -> Self {
        IndexResult {
            functions: HashMap::new(),
        }
    }

    pub fn add_function_call(&mut self, e: Entity) -> Result<()> {
        if e.get_kind() != clang::EntityKind::CallExpr {
            return Err(anyhow!("Not a function call expression"));
        }
        if let Some(ref_e) = e.get_reference() {
            if ref_e.get_kind() == clang::EntityKind::FunctionDecl {
                let usr = ref_e.get_usr().expect("function call without usr");
                self.functions
                    .entry(usr)
                    .or_insert(FunctionResult::new())
                    .calls
                    .insert(FileLocation::new(
                        e.get_location().expect("function call without location"),
                        ref_e.get_name().expect("function call without name").len() as u32,
                    ));
            }
        }
        Ok(())
    }

    pub fn add_function(&mut self, e: Entity) -> Result<()> {
        if e.get_kind() != clang::EntityKind::FunctionDecl {
            return Err(anyhow!("Not a function declaration"));
        }
        let loc = e
            .get_location()
            .expect("function declaration without location");
        if !loc.is_in_system_header() {
            let usr = e.get_usr().expect("function declaration without usr");
            let name = e.get_name().expect("function declaration without name");
            let func = self.functions.entry(usr).or_insert(FunctionResult::new());
            match (e.is_definition(), e.is_declaration()) {
                (false, true) => {
                    // function declaration
                    func.declarations
                        .insert(FileLocation::new(loc, name.len() as u32));
                }
                (false, false) => {
                    // function reference
                    return Err(anyhow!(
                        "Function reference found without declaration or definition"
                    ));
                }
                (true, _) => {
                    // function definition
                    func.definitions
                        .insert(FileLocation::new(loc, name.len() as u32));
                }
            }
        }
        Ok(())
    }

    pub fn get_functions(&self) -> &HashMap<Usr, FunctionResult> {
        &self.functions
    }
}

impl FileLocation {
    pub fn new(location: SourceLocation, len: u32) -> Self {
        let loc = location.get_expansion_location();
        FileLocation {
            file: loc.file.unwrap().get_path().to_str().unwrap().to_string(),
            loc: OneFileLocation {
                line: loc.line,
                column: loc.column,
                offset: loc.offset,
                len,
            },
        }
    }
}

impl FunctionResult {
    pub fn new() -> Self {
        FunctionResult {
            declarations: HashSet::new(),
            definitions: HashSet::new(),
            calls: HashSet::new(),
        }
    }
    pub fn declarations(&self) -> &HashSet<FileLocation> {
        &self.declarations
    }
    pub fn definitions(&self) -> &HashSet<FileLocation> {
        &self.definitions
    }
    pub fn calls(&self) -> &HashSet<FileLocation> {
        &self.calls
    }
}

/// 处理单个文件
pub fn main(file: &str, dir: &str, debug: bool) -> Result<()> {
    // let index = clang_sys::clang_createIndex(0, 0);
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
    let args: Vec<&'static str> = vec![];
    // Parse a source file into a translation unit
    let mut parser = index.parser(file);
    let parser = parser.arguments(args.as_slice());
    let parser = parser
        // .cache_completion_results(true)
        .detailed_preprocessing_record(true);
    let tu = parser.parse()?;
    let mut index_result = IndexResult::new();
    dfs(&tu.get_entity(), debug, &mut index_result);
    println!("Functions found: {:?}", index_result);
    let json = FileJson::from_index(&index_result, file.to_string())?;
    let json = serde_json::to_string(&json)?;
    println!("JSON output: {}", json);
    let file_path = format!("{}/{}.json", FILE_JSON_PREFIX, file);
    let path = std::path::Path::new(&file_path);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    let mut file_handle = fs::File::create(path)?;
    file_handle.write_all(json.as_bytes())?;
    dump_symbols(&index_result)?;
    Ok(())
}

fn dfs(entity: &Entity, debug: bool, ans: &mut IndexResult) {
    match entity.get_kind() {
        clang::EntityKind::CallExpr => {
            if let Err(e) = ans.add_function_call(entity.clone()) {
                eprintln!("Error adding function call: {}", e);
            }
        }
        clang::EntityKind::FunctionDecl => {
            if let Err(e) = ans.add_function(entity.clone()) {
                eprintln!("Error adding function: {}", e);
            }
        }
        _ => {}
    }
    if debug {
        println!("{:?}", &entity);
        println!("\t parent {:?}", entity.get_semantic_parent());
        println!(
            "\t definition {} declaration {}",
            entity.is_definition(),
            entity.is_declaration()
        );
        println!("\t definition {:?}", entity.get_definition());
        println!("\t reference {:?}", entity.get_reference());
        println!(
            "\t overridden methods {:?}",
            entity.get_overridden_methods()
        );
        // println!("\t mangled name {:?}", entity.get_mangled_name());
        println!("\t name {:?}", entity.get_name());
        // println!("\t name range {:?}", entity.get_name_ranges());
        println!("\t usr {:?}", entity.get_usr());
    }
    for child in entity.get_children() {
        dfs(&child, debug, ans);
    }
}

fn dump_symbols(index: &IndexResult) -> Result<()> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    let symbols_json = SymbolsJson::from_index(index)?;
    for symbol in symbols_json.symbols.iter() {
        let json = serde_json::to_string(&symbol)?;
        let file_path = format!(
            "{}/{}.json",
            SYMBOL_JSON_PREFIX,
            STANDARD.encode(symbol.name.as_bytes())
        );
        let path = std::path::Path::new(&file_path);
        let prefix = path.parent().unwrap();
        std::fs::create_dir_all(prefix).unwrap();
        let mut file_handle = fs::File::create(path)?;
        file_handle.write_all(json.as_bytes())?;
    }
    Ok(())
}
