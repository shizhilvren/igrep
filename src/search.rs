use crate::{
    builder::{AbsPath, Builder, FileContent, FileIndexBuilder},
    data::{FileData, FromToData, IndexData, NgramData},
    index::{FileIndex, FileLineIndex, LineIndex, NgramIndex},
    range::{FileRange, NgramRange},
};
use rayon::iter::*;
use regex_syntax::{
    hir::{Hir, HirKind, Literal},
    parse,
};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{Error, Read},
    mem::{replace, take},
};
use wasm_bindgen::prelude::*;

// Init -> NgramsIndex -> NgramData -> FileLinesIndex -> FileLinesData

#[wasm_bindgen]
pub struct Engine {
    index_data: IndexData,
}

#[wasm_bindgen]
pub struct OneSearchInit {
    tree: NgramTree,
}

#[wasm_bindgen]
pub struct OneSearchNgramsIndex {
    tree: NgramTree,
    indexs: Vec<NgramIndex>,
}

#[wasm_bindgen]
pub struct OneSearchNgramData {
    tree: NgramTree,
    indexs_ranges: Vec<(NgramIndex, NgramData)>,
}

#[wasm_bindgen]
pub struct OneSearchFileIndex {
    #[wasm_bindgen(readonly)]
    pub all: bool,
    #[wasm_bindgen(skip)]
    pub file_lines_index: HashMap<FileIndex, Vec<LineIndex>>,
}

#[wasm_bindgen]
pub struct OneSearchFileData {
    #[wasm_bindgen(skip)]
    pub file_resault: (FileData, Vec<LineIndex>),
}

#[derive(Debug)]
pub enum NgramTreeResult {
    ALL,
    Set(HashSet<FileLineIndex>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum NgramTree {
    Empty,
    ALL,
    Gram(NgramIndex),
    Concat(Vec<NgramTree>),
    Alternation(Vec<NgramTree>),
}

impl Engine {
    pub fn new(index_data_buf: Vec<u8>) -> Result<Self, Error> {
        let index_data = IndexData::from_data(index_data_buf)?;
        Ok(Self {
            index_data: index_data,
        })
    }

    pub fn init(&self, pattern: &str) -> Result<OneSearchInit, regex_syntax::Error> {
        let n = self.index_data.ngram_len();
        let hir = parse(pattern)?;
        let tree = Self::ngram_from_hir(&hir, n);
        Ok(OneSearchInit::new(tree))
    }

    pub fn init_to_ngram_index(&self, one_search_init: OneSearchInit) -> OneSearchNgramsIndex {
        OneSearchNgramsIndex::new(one_search_init.tree)
    }

    pub fn ngram_ranges(&self, one_search_ngram_index: &OneSearchNgramsIndex) -> Vec<NgramRange> {
        one_search_ngram_index
            .indexs
            .iter()
            .filter_map(|index| self.index_data.get_ngram_range(index))
            .collect::<Vec<_>>()
    }

    pub fn ngram_index_to_ngram_data(
        &self,
        one_search_ngram_index: OneSearchNgramsIndex,
        datas: Vec<Vec<u8>>,
    ) -> Result<OneSearchNgramData, Error> {
        let tree = one_search_ngram_index.tree;
        let datas = datas
            .into_iter()
            .map(|data| NgramData::from_data(data))
            .collect::<Result<Vec<_>, _>>()?;
        let indexs_ranges = one_search_ngram_index
            .indexs
            .into_iter()
            .zip(datas)
            .collect::<Vec<_>>();
        Ok(OneSearchNgramData {
            tree,
            indexs_ranges,
        })
    }

    pub fn ngram_data_to_file_lines(
        &self,
        one_search_ngram_data: OneSearchNgramData,
    ) -> OneSearchFileIndex {
        let index_data = one_search_ngram_data
            .indexs_ranges
            .into_iter()
            .collect::<HashMap<_, _>>();
        OneSearchFileIndex::new(one_search_ngram_data.tree.get_file_lines(&index_data))
    }

    pub fn show_info(&self) {
        self.index_data.show_info();
    }

    fn ngram_from_hir(hir: &Hir, n: u8) -> NgramTree {
        let kind = hir.kind();
        match kind {
            HirKind::Empty => NgramTree::ALL,
            HirKind::Literal(lit) => {
                let sub_tree = NgramIndex::from_str(lit.0.as_ref(), n)
                    .into_iter()
                    .map(|ngram| NgramTree::Gram(ngram))
                    .collect::<Vec<_>>();
                let len = sub_tree.len();
                match len {
                    0 => NgramTree::ALL,
                    1 => sub_tree.into_iter().nth(0).unwrap(),
                    _ => NgramTree::Concat(sub_tree),
                }
            }
            HirKind::Class(_) => NgramTree::ALL,
            HirKind::Look(_) => NgramTree::ALL,
            HirKind::Repetition(r) => match r.sub.kind() {
                HirKind::Literal(lit) => {
                    let lit = lit.0.as_ref();
                    let lit = std::iter::repeat_n(lit, r.min as usize)
                        .flatten()
                        .cloned()
                        .collect::<Vec<_>>();
                    let sub_tree = NgramIndex::from_str(lit.as_slice(), n)
                        .into_iter()
                        .map(|ngram| NgramTree::Gram(ngram))
                        .collect::<Vec<_>>();
                    NgramTree::Concat(sub_tree)
                }
                _ => NgramTree::ALL,
            },
            HirKind::Capture(c) => Engine::ngram_from_hir(c.sub.as_ref(), n),
            HirKind::Concat(hirs) => {
                let sub_tree = hirs
                    .iter()
                    .map(|hir| Engine::ngram_from_hir(hir, n))
                    .collect::<Vec<_>>();
                NgramTree::Concat(sub_tree)
            }
            HirKind::Alternation(hirs) => {
                let sub_tree = hirs
                    .iter()
                    .map(|hir| Engine::ngram_from_hir(hir, n))
                    .collect::<Vec<_>>();
                NgramTree::Alternation(sub_tree)
            }
        }
    }
}

impl NgramTree {
    fn is_all(&self) -> bool {
        match self {
            Self::ALL => true,
            Self::Empty => false,
            Self::Gram(_) => false,
            Self::Alternation(sub) => sub.iter().any(|ngram_tree| ngram_tree.is_all()),
            Self::Concat(sub) => sub.iter().all(|t| t.is_all()),
        }
    }
    pub fn ngrams(&self) -> Vec<NgramIndex> {
        let mut ngrams = match self {
            Self::ALL => vec![],
            Self::Empty => vec![],
            Self::Gram(e) => vec![e.clone()],
            Self::Alternation(sub) | Self::Concat(sub) => {
                sub.iter().map(|t| t.ngrams()).flatten().collect()
            }
        };
        ngrams.sort();
        ngrams.dedup();
        ngrams
    }
}

impl OneSearchInit {
    pub fn new(tree: NgramTree) -> Self {
        Self { tree }
    }
}

impl OneSearchNgramsIndex {
    pub fn new(tree: NgramTree) -> Self {
        let indexs = tree.ngrams();
        Self { tree, indexs }
    }
}

impl OneSearchFileIndex {
    pub fn new(ngram_tree_result: NgramTreeResult) -> Self {
        Self {
            all: matches!(ngram_tree_result, NgramTreeResult::ALL),
            file_lines_index: match ngram_tree_result {
                NgramTreeResult::ALL => HashMap::new(),
                NgramTreeResult::Set(set) => {
                    let mut file_lines_map: HashMap<FileIndex, HashSet<LineIndex>> = HashMap::new();
                    set.into_iter().for_each(|file_line_index| {
                        let file_index = file_line_index.file_id().to_owned();
                        let line_index = file_line_index.line_id().to_owned();
                        file_lines_map
                            .entry(file_index)
                            .or_default()
                            .insert(line_index);
                    });
                    file_lines_map
                        .into_iter()
                        .map(|(fid, lids)| (fid, lids.into_iter().collect::<Vec<_>>()))
                        .collect::<HashMap<_, _>>()
                }
            },
        }
    }
}

#[wasm_bindgen]
impl OneSearchFileIndex {
    pub fn file_range(&self) -> Vec<FileIndex> {
        self.file_lines_index
            .iter()
            .map(|(file_index, _)| file_index.clone())
            .collect::<Vec<_>>()
    }
    pub fn file_data(&self,idx:&IndexData, fid: &FileIndex, file_data: Vec<u8>) -> Option<OneSearchFileData> {
        if self.all {
            None
        }else{
            match self.file_lines_index.get(fid) {
                Some(lines) => Some(OneSearchFileData {
                    file_resault: (
                        idx.get_file_data(fid).unwrap_or(FileData::new("")),
                        lines.clone(),
                    ),
                }),
                None => None,
            }
        }
      
    }
}

impl NgramTreeResult {
    pub fn alternation(self: Self, b: Self) -> Self {
        match (self, b) {
            (Self::Set(sub1), Self::Set(sub2)) => {
                Self::Set(sub1.union(&sub2).cloned().collect::<HashSet<_>>())
            }
            (_, _) => Self::ALL,
        }
    }
    pub fn concat(self: Self, b: Self) -> Self {
        match (self, b) {
            (Self::Set(sub1), Self::Set(sub2)) => {
                Self::Set(sub1.intersection(&sub2).cloned().collect::<HashSet<_>>())
            }
            (Self::Set(sub1), Self::ALL) => Self::Set(sub1),
            (Self::ALL, Self::Set(sub2)) => Self::Set(sub2),
            (Self::ALL, Self::ALL) => Self::ALL,
        }
    }
}

impl NgramTree {
    pub fn get_file_lines(
        &self,
        ngram_to_data: &HashMap<NgramIndex, NgramData>,
    ) -> NgramTreeResult {
        let ans = match self {
            Self::ALL => NgramTreeResult::ALL,
            Self::Empty => NgramTreeResult::Set(HashSet::new()),
            Self::Gram(e) => match ngram_to_data.get(e) {
                Some(data) => NgramTreeResult::Set(
                    data.file_lines()
                        .clone()
                        .into_iter()
                        .collect::<HashSet<_>>(),
                ),
                _ => NgramTreeResult::Set(HashSet::new()),
            },
            Self::Alternation(sub) => sub
                .iter()
                .map(|t| t.get_file_lines(ngram_to_data))
                .fold(NgramTreeResult::Set(HashSet::new()), |ans, result| {
                    ans.alternation(result)
                }),

            Self::Concat(sub) => sub
                .iter()
                .map(|t| t.get_file_lines(ngram_to_data))
                .fold(NgramTreeResult::ALL, |ans, result| ans.concat(result)),
        };
        ans
    }
}
