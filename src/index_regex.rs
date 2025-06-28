use std::{collections::HashSet, io::Read};

use regex_syntax::{
    hir::{Hir, HirKind, Literal},
    parse,
};

use crate::index_builder::NgramIndex;

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

impl NgramTree {
    pub fn simple(self: Self) -> Self {
        println!("before simple {:?}", &self);
        let ans = match self {
            Self::Empty | Self::Gram(_) | Self::ALL => self,
            Self::Concat(sub_trees) => {
                assert!(!sub_trees.is_empty());
                sub_trees
                    .into_iter()
                    .map(|e| e.simple())
                    .fold(Self::ALL, |a, b| Self::simple_concat(a, b))
            }
            Self::Alternation(sub) => {
                assert!(!sub.is_empty());
                sub.into_iter()
                    .map(|e| e.simple())
                    .fold(Self::Empty, |a, b| Self::simple_alternation(a, b))
            }
        };
        println!("after simple {:?}", &ans);
        ans
    }
    fn simple_concat(a: Self, b: Self) -> Self {
        assert!(a.is_simple());
        assert!(b.is_simple());
        let a = a.get_base_concat();
        let b = b.get_base_concat();
        let mut union = a.intersection(&b).collect::<HashSet<_>>();
        if union.len() > 1 {
            union.remove(&Self::Empty);
        }
        let len = union.len();
        match len {
            0 => Self::Empty,
            1 => union.into_iter().nth(0).unwrap().clone(),
            _ => {
                if union.contains(&Self::ALL) {
                    Self::ALL
                } else {
                    Self::Alternation(union.into_iter().cloned().collect::<Vec<_>>())
                }
            }
        }
    }

    fn simple_alternation(a: Self, b: Self) -> Self {
        assert!(a.is_simple());
        assert!(b.is_simple());
        let a = a.get_base_concat();
        let b = b.get_base_concat();
        let mut union = a.union(&b).collect::<HashSet<_>>();
        if union.len() > 1 {
            union.remove(&Self::Empty);
        }
        let len = union.len();
        match len {
            0 => Self::Empty,
            1 => union.into_iter().nth(0).unwrap().clone(),
            _ => {
                if union.contains(&Self::ALL) {
                    Self::ALL
                } else {
                    Self::Alternation(union.into_iter().cloned().collect::<Vec<_>>())
                }
            }
        }
    }

    fn get_base_concat(self: Self) -> HashSet<Self> {
        assert!(self.is_simple());
        let mut ret = HashSet::new();
        match self {
            Self::ALL | Self::Empty | Self::Gram(_) | Self::Concat(_) => {
                ret.insert(self);
            }
            Self::Alternation(sub) => {
                ret = sub.into_iter().collect::<HashSet<_>>();
            }
        };
        ret
    }
    pub fn is_simple(&self) -> bool {
        match self {
            Self::ALL | Self::Empty | Self::Gram(_) => true,
            Self::Concat(_) => self.is_base_concat(),
            Self::Alternation(sub) => {
                let len = sub.len();
                match len {
                    0 | 1 => false,
                    _ => sub
                        .iter()
                        .all(|t| t.is_base_concat() && !t.is_all() && !t.is_empty()),
                }
            }
        }
    }

    pub fn is_base_concat(&self) -> bool {
        match self {
            Self::ALL | Self::Empty | Self::Gram(_) => true,
            Self::Alternation(_) => false,
            Self::Concat(sub) => {
                let len = sub.len();
                match len {
                    0 | 1 => false,
                    _ => sub.iter().all(|e| e.is_gram()),
                }
            }
        }
    }

    pub fn is_gram(&self) -> bool {
        match self {
            Self::Gram(_) => true,
            _ => false,
        }
    }

    pub fn is_base(&self) -> bool {
        match self {
            Self::ALL | Self::Empty | Self::Gram(_) => true,
            _ => false,
        }
    }

    pub fn is_all(&self) -> bool {
        match self {
            Self::ALL => true,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            _ => false,
        }
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
    fn test_ngram_tree_basic_types() {
        // 测试基本节点类型
        let empty = NgramTree::Empty;
        let all = NgramTree::ALL;
        let gram = NgramTree::Gram(NgramIndex::new("abc".as_bytes()));

        // 测试 Empty 节点
        assert!(empty.is_empty());
        assert!(!empty.is_all());
        assert!(empty.is_base());
        assert!(!empty.is_gram());

        // 测试 ALL 节点
        assert!(all.is_all());
        assert!(!all.is_empty());
        assert!(all.is_base());
        assert!(!all.is_gram());

        // 测试 Gram 节点
        assert!(gram.is_gram());
        assert!(!gram.is_empty());
        assert!(!gram.is_all());
        assert!(gram.is_base());
    }

    #[test]
    fn test_ngram_tree_concat() {
        // 创建一个简单的 Concat 节点
        let concat = NgramTree::Concat(vec![
            NgramTree::Gram(NgramIndex::new("abc".as_bytes())),
            NgramTree::Gram(NgramIndex::new("def".as_bytes())),
        ]);

        // 创建一个空的 Concat 节点
        let empty_concat = NgramTree::Concat(vec![]);

        // 创建一个包含非 Gram 节点的 Concat
        let mixed_concat = NgramTree::Concat(vec![
            NgramTree::Gram(NgramIndex::new("abc".as_bytes())),
            NgramTree::ALL,
        ]);

        assert!(!concat.is_base());
        assert!(concat.is_base_concat());
        assert!(!concat.is_gram());

        assert!(!empty_concat.is_base_concat());
        assert!(!mixed_concat.is_base_concat());
    }

    #[test]
    fn test_ngram_tree_alternation() {
        // 创建一个简单的 Alternation 节点
        let alt = NgramTree::Alternation(vec![
            NgramTree::Gram(NgramIndex::new("abc".as_bytes())),
            NgramTree::Gram(NgramIndex::new("def".as_bytes())),
        ]);

        // 验证属性
        assert!(!alt.is_base());
        assert!(!alt.is_base_concat());
        assert!(!alt.is_gram());
        assert!(!alt.is_empty());
        assert!(!alt.is_all());
    }

    #[test]
    fn test_ngram_tree_complex() {
        // 创建一个复杂的嵌套结构
        let complex = NgramTree::Concat(vec![
            NgramTree::Gram(NgramIndex::new("abc".as_bytes())),
            NgramTree::Alternation(vec![
                NgramTree::Gram(NgramIndex::new("def".as_bytes())),
                NgramTree::Empty,
            ]),
            NgramTree::ALL,
        ]);

        assert!(!complex.is_base());
        assert!(!complex.is_base_concat());
        assert!(!complex.is_simple());
    }

    #[test]
    fn test_simple_1() {
        // 创建一个复杂的嵌套结构
        let a = NgramTree::Gram(NgramIndex::new("abc".as_bytes()));
        let b = NgramTree::Gram(NgramIndex::new("def".as_bytes()));
        let c = NgramTree::Concat(vec![a, b]);
        let d = c.clone();
        assert_eq!(c, d.simple());
    }
}
