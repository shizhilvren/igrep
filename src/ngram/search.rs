use std::collections::HashMap;

use anyhow::{Result, anyhow};
use regex_syntax::{
    hir::{Hir, HirKind},
    parse,
};

use crate::ngram::{
    data::{GlobalData, NgramData},
    index::{FilesLinesIndex, NgramIndex, NgramIndexVec, SetCalculate},
};

pub struct SearchEngine {
    global_data: GlobalData,
}

pub struct SearchOneEngine {
    tree: NgramTree,
    re: regex::Regex,
}

pub struct SearchOneNgramResult {
    ngarm_to_data: HashMap<NgramIndex, NgramData>,
}
pub struct NgramIndexData {
    ngram: NgramIndex,
    data: NgramData,
}

#[derive(Debug)]
pub enum SearchOneFilesLinesResult {
    ALL,
    FilesLines(FilesLinesIndex),
}

#[derive(Clone)]
enum NgramTree {
    ALL,
    Gram(NgramIndex),
    Concat(Vec<NgramTree>),
    Alternation(Vec<NgramTree>),
}

impl SearchOneEngine {
    pub fn ngrams(&self) -> NgramIndexVec {
        self.tree.ngrams()
    }

    pub fn files_lines(&self, index_data: SearchOneNgramResult) -> SearchOneFilesLinesResult {
        self.tree.files_lines(&index_data)
    }
}

impl SearchEngine {
    pub fn search(&self, pattern: &str) -> Result<SearchOneEngine> {
        let n = self.global_data.ngram_len();
        let hir = parse(pattern).map_err(|e| anyhow!("parse error: {}", e))?;
        let tree = Self::ngram_from_hir(&hir, n);
        let re = regex::Regex::new(pattern).map_err(|e| anyhow!("regex error: {}", e))?;
        Ok(SearchOneEngine { tree: tree, re: re })
    }
}

impl SearchEngine {
    fn ngram_from_hir(hir: &Hir, n: u8) -> NgramTree {
        let kind = hir.kind();
        match kind {
            HirKind::Empty => NgramTree::ALL,
            HirKind::Literal(lit) => {
                let sub_tree = NgramIndexVec::from((lit.0.as_ref(), n))
                    .0
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
                    let sub_tree = NgramIndexVec::from((lit.as_slice(), n))
                        .0
                        .into_iter()
                        .map(|ngram| NgramTree::Gram(ngram))
                        .collect::<Vec<_>>();
                    NgramTree::Concat(sub_tree)
                }
                _ => NgramTree::ALL,
            },
            HirKind::Capture(c) => SearchEngine::ngram_from_hir(c.sub.as_ref(), n),
            HirKind::Concat(hirs) => {
                let sub_tree = hirs
                    .iter()
                    .map(|hir| SearchEngine::ngram_from_hir(hir, n))
                    .collect::<Vec<_>>();
                NgramTree::Concat(sub_tree)
            }
            HirKind::Alternation(hirs) => {
                let sub_tree = hirs
                    .iter()
                    .map(|hir| SearchEngine::ngram_from_hir(hir, n))
                    .collect::<Vec<_>>();
                NgramTree::Alternation(sub_tree)
            }
        }
    }
}

impl NgramTree {
    fn ngrams(&self) -> NgramIndexVec {
        let ngrams = match self {
            Self::ALL => vec![],
            Self::Gram(ngram) => vec![ngram.clone()],
            Self::Alternation(sub) | Self::Concat(sub) => sub
                .iter()
                .map(|t| t.ngrams().0)
                .flatten()
                .collect::<Vec<_>>(),
        };
        NgramIndexVec::from(ngrams)
    }

    fn files_lines(&self, index_data: &SearchOneNgramResult) -> SearchOneFilesLinesResult {
        let ans = match self {
            Self::ALL => SearchOneFilesLinesResult::ALL,
            Self::Gram(e) => match index_data.ngram_to_data(e) {
                Some(data) => SearchOneFilesLinesResult::FilesLines(data.files_lines().clone()),
                _ => SearchOneFilesLinesResult::FilesLines(FilesLinesIndex::from(vec![])),
            },
            Self::Alternation(sub) => sub.iter().map(|t| t.files_lines(index_data)).fold(
                SearchOneFilesLinesResult::FilesLines(FilesLinesIndex::from(vec![])),
                |ans, result| ans.alternation(result),
            ),

            Self::Concat(sub) => sub
                .iter()
                .map(|t| t.files_lines(index_data))
                .fold(SearchOneFilesLinesResult::ALL, |ans, result| {
                    ans.concat(result)
                }),
        };
        ans
    }
}

impl SearchOneFilesLinesResult{
    pub fn files_lines_index(&self) -> Option<&FilesLinesIndex> {
        match self {
            SearchOneFilesLinesResult::ALL => None,
            SearchOneFilesLinesResult::FilesLines(files_lines_index) => Some(files_lines_index),
        }
    }
}

impl SearchOneFilesLinesResult {
    fn alternation(self, other: Self) -> Self {
        match (self, other) {
            (SearchOneFilesLinesResult::ALL, _) | (_, SearchOneFilesLinesResult::ALL) => {
                SearchOneFilesLinesResult::ALL
            }
            (
                SearchOneFilesLinesResult::FilesLines(a),
                SearchOneFilesLinesResult::FilesLines(b),
            ) => SearchOneFilesLinesResult::FilesLines(FilesLinesIndex::union(a, b)),
        }
    }

    fn concat(self, other: Self) -> Self {
        match (self, other) {
            (SearchOneFilesLinesResult::ALL, r) | (r, SearchOneFilesLinesResult::ALL) => r,
            (
                SearchOneFilesLinesResult::FilesLines(a),
                SearchOneFilesLinesResult::FilesLines(b),
            ) => SearchOneFilesLinesResult::FilesLines(FilesLinesIndex::intersection(a, b)),
        }
    }
}

impl SearchOneNgramResult {
    fn ngram_to_data(&self, ngram: &NgramIndex) -> Option<&NgramData> {
        self.ngarm_to_data.get(ngram)
    }
}

impl From<GlobalData> for SearchEngine {
    fn from(global_data: GlobalData) -> Self {
        Self { global_data }
    }
}

impl From<Vec<NgramIndexData>> for SearchOneNgramResult {
    fn from(value: Vec<NgramIndexData>) -> Self {
        let ngarm_to_data = value
            .into_iter()
            .map(|data| (data.ngram, data.data))
            .collect::<HashMap<_, _>>();
        SearchOneNgramResult { ngarm_to_data }
    }
}

impl From<(NgramIndex, NgramData)> for NgramIndexData {
    fn from(value: (NgramIndex, NgramData)) -> Self {
        NgramIndexData {
            ngram: value.0,
            data: value.1,
        }
    }
}
