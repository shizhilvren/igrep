use std::collections::HashMap;

use anyhow::{Result, anyhow};
use regex_syntax::{
    hir::{Hir, HirKind},
    parse,
};

use crate::ngram::{
    builder::AbsPath,
    data::{FileData, GlobalData, NgramData},
    index::{FileLinesIndex, FilesLinesIndex, LinesIndex, NgramIndex, NgramIndexVec, SetCalculate},
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

#[derive(Debug)]
pub struct SearchOneFileLinesContentResult {
    full_file_name: String,
    lines: Vec<SearchOneLineContentResult>,
}
#[derive(Debug)]
pub struct SearchOneLineContentResult {
    line_num: u32,
    content: String,
    match_range: Vec<(u32, u32)>,
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

    pub fn file_lines_match(
        &self,
        file_data: &FileData,
        lines_index: &LinesIndex,
    ) -> Result<SearchOneFileLinesContentResult> {
        let full_file_name = file_data.full_file_name().to_string();
        let lines = lines_index
            .lines()
            .iter()
            .filter_map(|line_index| {
                let content = file_data
                    .lines(line_index)
                    .map_or_else(
                        || {
                            Err(anyhow!(
                                "file {full_file_name} not have line {}",
                                line_index.line_num()
                            ))
                        },
                        |line| Ok(line),
                    )
                    .and_then(|content| {
                        let match_range = self
                            .re
                            .find_iter(content.as_str())
                            .map(|m| (m.start() as u32, m.end() as u32))
                            .collect::<Vec<_>>();
                        Ok(SearchOneLineContentResult {
                            line_num: line_index.line_num(),
                            content: content.to_string(),
                            match_range,
                        })
                    });
                content.map_or_else(
                    |e| Some(Err(e)),
                    |content| {
                        if content.is_empty() {
                            None
                        } else {
                            Some(Ok(content))
                        }
                    },
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(SearchOneFileLinesContentResult {
            full_file_name,
            lines,
        })
    }
}

impl SearchOneFileLinesContentResult {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn full_file_name(&self) -> &String {
        &self.full_file_name
    }

    pub fn lines(&self) -> &[SearchOneLineContentResult] {
        &self.lines
    }
}

impl SearchOneLineContentResult {
    pub fn is_empty(&self) -> bool {
        self.match_range.is_empty()
    }

    pub fn line_num(&self) -> u32 {
        self.line_num
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn match_range(&self) -> &[(u32, u32)] {
        &self.match_range
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

impl SearchOneFilesLinesResult {
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

#[cfg(test)]
mod tests {
    use super::SearchOneFilesLinesResult;
    use crate::ngram::index::{FileIndex, FileLinesIndex, FilesLinesIndex, LineIndex, LinesIndex};

    fn files_lines(entries: &[(u32, &[u32])]) -> FilesLinesIndex {
        let files = entries
            .iter()
            .map(|(file_id, lines)| {
                let lines = lines
                    .iter()
                    .map(|line| LineIndex::from(*line))
                    .collect::<Vec<_>>();
                FileLinesIndex::from((FileIndex::from(*file_id), LinesIndex::from(lines)))
            })
            .collect::<Vec<_>>();
        FilesLinesIndex::from(files)
    }

    fn flatten(index: &FilesLinesIndex) -> Vec<(u32, Vec<u32>)> {
        index
            .files_lines()
            .iter()
            .map(|file_lines| {
                (
                    file_lines.file_id().file_id(),
                    file_lines
                        .lines_index()
                        .lines()
                        .iter()
                        .map(|line| line.line_id())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>()
    }

    #[test]
    fn alternation_returns_all_when_any_side_is_all() {
        let left = SearchOneFilesLinesResult::ALL;
        let right = SearchOneFilesLinesResult::FilesLines(files_lines(&[(1, &[1, 2])]));
        let result = left.alternation(right);
        assert!(matches!(result, SearchOneFilesLinesResult::ALL));

        let left = SearchOneFilesLinesResult::FilesLines(files_lines(&[(2, &[3])]));
        let right = SearchOneFilesLinesResult::ALL;
        let result = left.alternation(right);
        assert!(matches!(result, SearchOneFilesLinesResult::ALL));
    }

    #[test]
    fn alternation_unions_files_and_lines() {
        let left = SearchOneFilesLinesResult::FilesLines(files_lines(&[(1, &[0, 2]), (3, &[5])]));
        let right = SearchOneFilesLinesResult::FilesLines(files_lines(&[(2, &[4])]));

        let result = left.alternation(right);

        match result {
            SearchOneFilesLinesResult::ALL => panic!("expected FilesLines result"),
            SearchOneFilesLinesResult::FilesLines(index) => {
                let actual = flatten(&index);
                let expected = vec![(1, vec![0, 2]), (2, vec![4]), (3, vec![5])];
                assert_eq!(actual, expected);
            }
        }
    }
}
