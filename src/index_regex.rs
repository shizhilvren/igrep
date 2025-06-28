use std::{
    collections::{HashMap, HashSet},
    io::Read,
    result,
};

use regex_syntax::{
    hir::{Hir, HirKind, Literal},
    parse,
};

use crate::{
    index_builder::{FileLineIndex, NgramIndex},
    index_file::NgramData,
};

pub struct Engine {
    regex: Hir,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NgramTree {
    Empty,
    ALL,
    Gram(NgramIndex),
    Concat(Vec<NgramTree>),
    Alternation(Vec<NgramTree>),
}

#[derive(Debug)]
pub enum NgramTreeResult {
    ALL,
    Set(HashSet<FileLineIndex>),
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
    pub fn is_all(&self) -> bool {
        match self {
            Self::ALL => true,
            Self::Empty => false,
            Self::Gram(_) => false,
            Self::Alternation(sub) => sub.iter().any(|ngram_tree| ngram_tree.is_all()),
            Self::Concat(sub) => sub.iter().all(|t| t.is_all()),
        }
    }
    pub fn ngrams(&self) -> HashSet<NgramIndex> {
        match self {
            Self::ALL => HashSet::new(),
            Self::Empty => HashSet::new(),
            Self::Gram(e) => vec![e.clone()].into_iter().collect::<HashSet<_>>(),
            Self::Alternation(sub) | Self::Concat(sub) => {
                sub.iter().map(|t| t.ngrams()).flatten().collect()
            }
        }
    }

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
                .fold(NgramTreeResult::ALL, |ans, result| {
                    ans.concat(result)
                }),
        };
        ans
    }
}

impl Engine {
    pub fn new(pattern: &str) -> Result<Self, regex_syntax::Error> {
        let regex = parse(pattern)?;
        Ok(Self { regex })
    }

    pub fn ngram(&self, n: usize) -> NgramTree {
        Self::ngram_from_hir(&self.regex, n)
    }

    fn ngram_from_hir(hir: &Hir, n: usize) -> NgramTree {
        let ngram_tree = NgramTree::Empty;
        let kind = hir.kind();
        match kind {
            HirKind::Empty => NgramTree::ALL,
            HirKind::Literal(lit) => {
                let mut sub_tree = NgramIndex::from_str(lit.0.as_ref(), n)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ngram_tree_basic_types() -> Result<(), Box<dyn std::error::Error>> {
        let engine = Engine::new("ab.*cd")?;
        let a = engine.ngram(3);
        assert!(a.is_all());
        Ok(())
    }

    #[test]
    fn test_1() -> Result<(), Box<dyn std::error::Error>> {
        let engine = Engine::new("abcd.*cwerfasdfd")?;
        let a = engine.ngram(3);
        assert!(!a.is_all());
        Ok(())
    }

    #[test]
    fn test_2() -> Result<(), Box<dyn std::error::Error>> {
        let engine = Engine::new("abcd.*cwer|fasdfd")?;
        let a = engine.ngram(3);
        assert!(!a.is_all());
        Ok(())
    }
    #[test]
    fn test_3() -> Result<(), Box<dyn std::error::Error>> {
        let engine = Engine::new("[abcd.*cwer|fasdfd]{1,2}")?;
        let a = engine.ngram(3);
        assert!(!a.is_all());
        Ok(())
    }
}
