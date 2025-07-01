use crate::{
    encode,
    index::{FileIndex, FileLineIndex, LineIndex, NgramIndex},
};
use std::{
    collections::HashMap,
    fs::{self},
    hash::Hash,
    io::{self, Error},
};

pub struct FileIndexBuilder {
    file_to_id: HashMap<AbsPath, FileIndex>,
}

pub struct FileIndexFinalBuilder(FileIndexBuilder);

pub struct Buuilder {}

impl Buuilder {
    pub fn index(
        file_index_builder: &FileIndexFinalBuilder,
        file_content: FileContent,
        ngram_len: u8,
    ) -> Option<(FileIndex, FileContent, HashMap<NgramIndex, Vec<LineIndex>>)> {
        let mut ngram_to_file_line: HashMap<NgramIndex, Vec<LineIndex>> = HashMap::new();
        let name = file_content.get_name();
        file_index_builder.get(name).and_then(|fid| {
            file_content
                .lines()
                .iter()
                .map(|line| NgramIndex::from_str(line.as_bytes(), ngram_len))
                .enumerate()
                .map(|(id, ngrams)| (LineIndex::new((id + 1) as u32), ngrams))
                .for_each(|(lid, ngrams)| {
                    ngrams.into_iter().for_each(|ngram| {
                        ngram_to_file_line.entry(ngram).or_insert(vec![]).push(lid);
                    })
                });
            Some((*fid, file_content, ngram_to_file_line))
        })
    }
    /// please make sure FileIndex not have same one
    pub fn merge(
        file_ngrams: Vec<(FileIndex, FileContent, HashMap<NgramIndex, Vec<LineIndex>>)>,
    ) -> encode::Encode {
        let (fid_to_content, ngram_fid_lid): (Vec<_>, Vec<_>) = file_ngrams
            .into_iter()
            .map(|(fid, file_content, ngrams)| {
                (
                    (fid, file_content),
                    ngrams
                        .into_iter()
                        .map(|(ngram, lids)| {
                            (
                                ngram,
                                lids.into_iter()
                                    .map(|lid| FileLineIndex::new(fid, lid))
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .unzip();
        let mut ngram_to_fid_lid: HashMap<NgramIndex, Vec<FileLineIndex>> = HashMap::new();

        ngram_fid_lid
            .into_iter()
            .flatten()
            .for_each(|(ngram, fid_lids)| {
                fid_lids.into_iter().for_each(|fid_lid| {
                    ngram_to_fid_lid
                        .entry(ngram.clone())
                        .or_insert(vec![])
                        .push(fid_lid);
                });
            });
        encode::Encode::new(fid_to_content, ngram_to_fid_lid)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct AbsPath {
    path: String,
}

pub struct FileContent {
    full_file_name: AbsPath,
    lines: Vec<String>,
}

impl FileContent {
    /// creates a new `FileContent` from a file name.
    pub fn new(file_name: AbsPath, lines: Vec<String>) -> Self {
        Self {
            full_file_name: file_name,
            lines,
        }
    }
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// # panic
    pub fn get_line(&self, line_number: LineIndex) -> Option<&String> {
        match line_number.line_number() {
            0 => None, // Lines are 1-indexed
            _ => self
                .lines
                .get(line_number.line_number().saturating_sub(1) as usize),
        }
    }
    pub fn get_name(&self) -> &AbsPath {
        &self.full_file_name
    }
}

// impl IndexBuilder {
//     pub fn new(path: String) -> Result<Self, io::Error> {
//         Ok(Self {
//             path: AbsPath::new(path)?,
//             index: Some(Index::default()),
//             file_to_id: FileIDBuilder::default(),
//         })
//     }
//     /// # panic
//     /// Panic if the 'file' is already indexed.
//     /// Panic if `n` is zero.
//     pub fn index(&mut self, file: String, n: u8) -> Result<(), Error> {
//         let file_content = FileContent::new(file)?;
//         let file_index = self.file_to_id.get_or_insert(file_content.get_name());
//         self.index
//             .as_mut()
//             .and_then(|idx| {
//                 idx.index(file_index, file_content, n)
//                     .map_or(None, |_| Some(()))
//             })
//             .ok_or({
//                 Error::new(
//                     io::ErrorKind::AlreadyExists,
//                     format!("Failed to index file",),
//                 )
//             })?;
//         Ok(())
//     }
//     pub fn dump(&mut self) -> Result<(), Error> {
//         let mut data = Data::new(self.index.take().unwrap());
//         data.dump()?;
//         Ok(())
//     }
// }

impl AbsPath {
    pub fn new(path: String) -> Result<Self, io::Error> {
        Ok(Self {
            path: fs::canonicalize(path)?.to_string_lossy().into_owned(),
        })
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

impl FileIndexBuilder {
    pub fn new() -> Self {
        Self {
            file_to_id: HashMap::new(),
        }
    }

    pub fn insert(&mut self, path: &AbsPath) -> Result<(), io::Error> {
        let new_id = self.file_to_id.len() as u32;
        match self.file_to_id.contains_key(path) {
            true => Err(Error::new(
                io::ErrorKind::AlreadyExists,
                format!("File with path {} is already indexed", path.path()),
            )),
            false => {
                self.file_to_id.insert(path.clone(), FileIndex::new(new_id));
                Ok(())
            }
        }
    }
    pub fn make_final(self) -> FileIndexFinalBuilder {
        FileIndexFinalBuilder(self)
    }
}

impl FileIndexFinalBuilder {
    pub fn files(&self) -> Vec<(&FileIndex, &AbsPath)> {
        self.0
            .file_to_id
            .iter()
            .map(|(path, id)| (id, path))
            .collect()
    }
    pub fn get(&self, path: &AbsPath) -> Option<&FileIndex> {
        self.0.file_to_id.get(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    // #[test]
    // fn test_index() {
    //     let mut index_builder = IndexBuilder::new("test".to_string()).unwrap();
    //     index_builder.index("test/file".to_string(), 3_u8).unwrap();
    //     assert_eq!(
    //         index_builder
    //             .index
    //             .unwrap()
    //             .ngram_to_file_line
    //             .get(&NgramIndex::new(&[51, 52, 53]))
    //             .unwrap(),
    //         &vec![FileLineIndex {
    //             file_id: FileIndex { file_id: 0 },
    //             line_id: LineIndex { line: 1 }
    //         }]
    //     );
    // }
    #[test]
    fn test_ngram_index_from_str() {
        let ngrams = NgramIndex::from_str("hello".as_bytes(), 3);
        let ngrams = ngrams.into_iter().collect::<HashSet<_>>();
        assert_eq!(ngrams.len(), 3);
        let expected = HashSet::from([
            NgramIndex::new("hel".as_bytes()),
            NgramIndex::new("ell".as_bytes()),
            NgramIndex::new("llo".as_bytes()),
        ]);
        assert_eq!(ngrams, expected);
    }

    #[test]
    #[should_panic(expected = "Ngram cannot be empty")]
    fn test_ngram_index_new_empty() {
        NgramIndex::new(&[]);
    }

    #[test]
    fn test_ngram_index_new() {
        let ngram = NgramIndex::new(b"abc");
        assert_eq!(ngram, NgramIndex::new("abc".as_bytes()));
    }

    // #[test]
    // fn test_file_content_new() {
    //     let file_content = FileContent::new(
    //         AbsPath::new("test/textfile"),
    //         vec![String::from_str("abc"), String::from_str("defasdf")],
    //     );
    //     assert!(file_content.is_ok());
    //     let content = file_content.unwrap();
    //     assert!(!content.lines().is_empty());
    //     assert!(content.get_line(LineIndex::new(0)).is_none());
    //     assert!(content.get_line(LineIndex::new(1)).is_some());
    // }

    // #[test]
    // fn test_file_line() {
    //     let file_content = FileContent::new(
    //         AbsPath::new("test/textfile"),
    //         vec![String::from_str("123456789"), String::from_str("98765")],
    //     );
    //     assert!(file_content.is_ok());
    //     let content = file_content.unwrap();
    //     assert_eq!(content.get_line(LineIndex::new(2)).unwrap(), "98765");
    // }

    // #[test]
    // fn test_abs_path_new() {
    //     let abs_path = AbsPath::new("test/../test/textfile".to_string());
    //     let path = abs_path.unwrap();
    //     assert!(PathBuf::from(path.path()).is_absolute());
    // }

    // #[test]
    // fn test_index_builder_new() {
    //     let index_builder = IndexBuilder::new("test/textfile".to_string());
    //     assert!(index_builder.is_ok());
    //     let builder = index_builder.unwrap();
    //     let abs_path = env::current_dir().unwrap().join("test/textfile");
    //     assert_eq!(builder.path.path(), abs_path.to_str().unwrap());
    // }
    // #[test]
    // fn test_file_id_builder_make_or_insert() {
    //     let mut file_id_builder = FileIDBuilder::default();
    //     let abs_path = AbsPath::new("test/textfile".to_string()).unwrap();
    //     let file_index = file_id_builder.get_or_insert(&abs_path);
    //     assert_eq!(file_index.file_id, 0);
    //     let file_index2 = file_id_builder.get_or_insert(&abs_path);
    //     assert_eq!(file_index2.file_id, 0);
    //     assert_eq!(file_id_builder.file_to_id.len(), 1);
    // }
}
