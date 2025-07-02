use crate::{
    builder::{AbsPath, Builder, FileContent, FileIndexBuilder},
    data::{FromToData, IndexData},
    index::{FileIndex, FileLineIndex, LineIndex, NgramIndex},
    range::NgramRange,
};
use regex_syntax::{
    hir::{Hir, HirKind, Literal},
    parse,
};
use std::{
    collections::{HashMap, HashSet},
    io::{Error, Read},
    result,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Engine {
    index_data: IndexData,
    one_search: Option<OneSearch>,
}

struct OneSearch {
    tree: NgramTree,
    ngram_indexs: Option<Vec<NgramIndex>>,
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
            one_search: None,
        })
    }

    pub fn regex(&mut self, pattern: &str) -> Result<(), regex_syntax::Error> {
        let n = self.index_data.ngram_len();
        let hir = parse(pattern)?;
        let tree = Self::ngram_from_hir(&hir, n);
        self.one_search = Some(OneSearch::new(tree));
        Ok(())
    }

    pub fn ngram_ranges(&mut self) -> Option<Vec<NgramRange>> {
        self.one_search
            .as_mut()
            .and_then(|one_search| Some(one_search.ngram_ranges(&self.index_data)))
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

impl OneSearch {
    pub fn new(tree: NgramTree) -> Self {
        Self {
            tree,
            ngram_indexs: None,
        }
    }
    pub fn ngram_ranges(&mut self, idx: &IndexData) -> Vec<NgramRange> {
        let ngrams_indexs = self.tree.ngrams();
        let (indexs, rangs) = ngrams_indexs
            .into_iter()
            .filter_map(|index| idx.get_ngram_range(&index).and_then(|r| Some((index, r))))
            .unzip();
        self.ngram_indexs = Some(indexs);
        rangs
    }
}
