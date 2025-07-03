use crate::{
    data::{FileData, FileLineData, FromToData, IndexData, NgramData},
    index::{FileIndex, FileLineIndex, LineIndex, NgramIndex},
    range::{FileRange, NgramRange},
};
use regex_syntax::{
    hir::{Hir, HirKind},
    parse,
};
use std::collections::{HashMap, HashSet};
use wasm_bindgen::prelude::*;

// Init -> NgramsIndex -> NgramData -> FileLinesIndex -> FileLinesData

#[wasm_bindgen]
pub struct Engine {
    index_data: IndexData,
}

#[derive(Debug)]
pub enum NgramTreeResult {
    ALL,
    Set(HashSet<FileLineIndex>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum NgramTree {
    ALL,
    Gram(NgramIndex),
    Concat(Vec<NgramTree>),
    Alternation(Vec<NgramTree>),
}

#[wasm_bindgen]
pub struct NgramTreeStruct {
    #[wasm_bindgen(skip)]
    tree: NgramTree,
}

#[wasm_bindgen]
pub struct NgramTreeResultStruct {
    #[wasm_bindgen(skip)]
    result: NgramTreeResult,
}

#[wasm_bindgen]
pub struct FileLines {
    #[wasm_bindgen(readonly)]
    pub file: FileIndex,
    lines: Vec<LineIndex>,
}

#[wasm_bindgen]
pub struct NgreamIndexRange {
    #[wasm_bindgen(skip)]
    index: NgramIndex,
    #[wasm_bindgen(readonly)]
    pub range: NgramRange,
}

#[wasm_bindgen]
pub struct NgreamIndexData {
    #[wasm_bindgen(skip)]
    index: NgramIndex,
    #[wasm_bindgen(skip)]
    data: Vec<u8>,
}

#[wasm_bindgen]
impl NgreamIndexData {
    #[wasm_bindgen(constructor)]
    pub fn new(idx: NgramIndex, data: Vec<u8>) -> Self {
        Self {
            index: idx,
            data: data,
        }
    }
}

#[wasm_bindgen]
impl NgreamIndexRange {
    #[wasm_bindgen(constructor)]
    pub fn new(idx: NgramIndex, range: NgramRange) -> Self {
        Self {
            index: idx,
            range: range,
        }
    }
    pub fn index(&self) -> NgramIndex {
        self.index.clone()
    }
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(index_data_buf: Vec<u8>) -> Result<Self, String> {
        let index_data = IndexData::from_data(index_data_buf).map_err(|e| format!("{}", e))?;
        Ok(Self { index_data })
    }

    pub fn regex(&self, pattern: &str) -> Result<NgramTreeStruct, String> {
        let n = self.index_data.ngram_len();
        let hir = parse(pattern).map_err(|e| format!("{}", e))?;
        let tree = Self::ngram_from_hir(&hir, n);
        Ok(NgramTreeStruct { tree: tree })
    }

    pub fn ngram_ranges(&self, tree: &NgramTreeStruct) -> Vec<NgreamIndexRange> {
        tree.tree
            .ngrams()
            .iter()
            .filter_map(|index| {
                self.index_data
                    .get_ngram_range(index)
                    .and_then(|range| Some(NgreamIndexRange::new(index.clone(), range)))
            })
            .collect::<Vec<_>>()
    }

    pub fn get_search_result(
        &self,
        tree: &NgramTreeStruct,
        ngram_to_data: Vec<NgreamIndexData>,
    ) -> Result<NgramTreeResultStruct, String> {
        let ngram_to_data = ngram_to_data
            .into_iter()
            .map(|idx_data| {
                NgramData::from_data(idx_data.data).and_then(|data| Ok((idx_data.index, data)))
            })
            .collect::<Result<HashMap<_, _>, _>>()
            .map_err(|e| format!("{}", e))?;
        Ok(NgramTreeResultStruct {
            result: tree.tree.get_file_lines(&ngram_to_data),
        })
    }

    pub fn file_range(&self, fid: &FileIndex) -> Option<FileRange> {
        self.index_data.get_file_range(fid)
    }

    pub fn build_file_data(&self, data: Vec<u8>) -> Result<FileData, String> {
        FileData::from_data(data).map_err(|e| format!("{}", e))
    }

    pub fn build_file_line_data(&self, data: Vec<u8>) -> Result<FileLineData, String> {
        FileLineData::from_data(data).map_err(|e| format!("{}", e))
    }
}

impl Engine {
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

#[wasm_bindgen]
impl NgramTreeResultStruct {
    pub fn is_all(&self) -> bool {
        return matches!(self.result, NgramTreeResult::ALL);
    }
    pub fn file_lines(&self) -> Result<Vec<FileLines>, String> {
        match &self.result {
            NgramTreeResult::ALL => Err("result is ALL".to_string()),
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
                Ok(file_lines_map
                    .into_iter()
                    .map(|(fid, lids)| {
                        let mut lids = lids.into_iter().collect::<Vec<_>>();
                        lids.sort();
                        (fid, lids)
                    })
                    .map(|(fid, lids)| FileLines {
                        file: fid,
                        lines: lids,
                    })
                    .collect::<Vec<_>>())
            }
        }
    }
}

#[wasm_bindgen]
impl FileLines {
    pub fn lines(&self) -> Vec<LineIndex> {
        self.lines.clone()
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
    #[allow(unused)]
    fn is_all(&self) -> bool {
        match self {
            Self::ALL => true,
            Self::Gram(_) => false,
            Self::Alternation(sub) => sub.iter().any(|ngram_tree| ngram_tree.is_all()),
            Self::Concat(sub) => sub.iter().all(|t| t.is_all()),
        }
    }
    pub fn ngrams(&self) -> Vec<NgramIndex> {
        let mut ngrams = match self {
            Self::ALL => vec![],
            Self::Gram(e) => vec![e.clone()],
            Self::Alternation(sub) | Self::Concat(sub) => {
                sub.iter().map(|t| t.ngrams()).flatten().collect()
            }
        };
        ngrams.sort();
        ngrams.dedup();
        ngrams
    }
    pub fn get_file_lines(
        &self,
        ngram_to_data: &HashMap<NgramIndex, NgramData>,
    ) -> NgramTreeResult {
        let ans = match self {
            Self::ALL => NgramTreeResult::ALL,
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
