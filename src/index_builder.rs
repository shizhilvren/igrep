use std::{
    collections::HashMap,
    fs,
    hash::Hash,
    io::{self, Error},
};

pub struct IndexBuilder {
    pub(crate) path: AbsPath,
    index: Index,
    file_to_id: FileIDBuilder,
}

#[derive(Default)]
struct FileIDBuilder {
    file_to_id: HashMap<AbsPath, FileIndex>,
}

#[derive(Default)]
struct Index {
    id_to_file: HashMap<FileIndex, FileContent>,
    ngram_to_file_line: HashMap<NgramIndex, Vec<FileLine>>,
}
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct AbsPath {
    path: String,
}

struct FileContent {
    full_file_name: AbsPath,
    lines: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate)  struct FileIndex {
    file_id: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate)  struct LineIndex {
    line: u32,
}
#[derive(Clone, PartialEq, Debug)]
pub(crate)  struct FileLine {
    file_id: FileIndex,
    line_id: LineIndex,
}

/// This is NgramIndex, which is used to represent the index of n-grams in a file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate)  enum NgramIndex {
    Last(u8),
    Char(Box<(u8, NgramIndex)>),
}

impl NgramIndex {
    /// # Panics
    ///
    /// Panics if `n` size is zero.
    pub fn from_str(s: &str, n: usize) -> Vec<NgramIndex> {
        s.as_bytes()
            .windows(n)
            .map(|ngram| NgramIndex::new(ngram))
            .collect::<Vec<_>>()
    }
    /// # Panics
    ///
    /// Panics if `ngram` size is zero.
    pub fn new(ngram: &[u8]) -> Self {
        match ngram.len() {
            0 => panic!("Ngram cannot be empty"),
            1 => NgramIndex::Last(ngram[0]),
            _ => NgramIndex::Char(Box::new((ngram[0], NgramIndex::new(&ngram[1..])))),
        }
    }
}

impl FileContent {
    /// creates a new `FileContent` from a file name.
    pub fn new(file_name: String) -> Result<Self, io::Error> {
        let path = fs::canonicalize(&file_name)?;
        let lines = fs::read_to_string(&path)?
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        Ok(Self {
            full_file_name: AbsPath::new(file_name)?,
            lines,
        })
    }
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// # panic
    pub fn get_line(&self, line_number: usize) -> Option<&String> {
        match line_number {
            0 => None, // Lines are 1-indexed
            _ => self.lines.get(line_number - 1),
        }
    }
    pub fn get_name(&self) -> &AbsPath {
        &self.full_file_name
    }
}

impl IndexBuilder {
    pub fn new(path: String) -> Result<Self, io::Error> {
        Ok(Self {
            path: AbsPath::new(path)?,
            index: Index::default(),
            file_to_id: FileIDBuilder::default(),
        })
    }
    /// # panic
    /// Panic if the 'file' is already indexed.
    /// Panic if `n` is zero.
    pub fn index(&mut self, file: String, n: usize) -> Result<(), Error> {
        let file_content = FileContent::new(file)?;
        let file_index = self.file_to_id.get_or_insert(file_content.get_name());
        self.index.index(file_index, file_content, n).map_err(|e| {
            Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Failed to index file: {}", e),
            )
        })?;
        Ok(())
    }
}

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

impl Index {
    /// # panic
    /// Panic if the `file_id` is found in the index.
    /// Panic if the ` n` is zero.
    fn index(
        &mut self,
        file_id: &FileIndex,
        file_content: FileContent,
        n: usize,
    ) -> Result<(), String> {
        let ans = self.id_to_file.insert(file_id.clone(), file_content);
        if (ans.is_some()) {
            return Err(format!(
                "File with id {} is already indexed",
                file_id.file_id
            ));
        }
        let file_content = self.id_to_file.get(file_id).unwrap();
        file_content
            .lines()
            .iter()
            .enumerate()
            .map(|(id, line)| (id + 1, line))
            .map(|(line_id, line)| {
                let file_line = FileLine {
                    file_id: file_id.clone(),
                    line_id: LineIndex {
                        line: line_id as u32,
                    },
                };
                let ngrams = NgramIndex::from_str(line.as_str(), n);
                (file_line, ngrams)
            })
            .for_each(|(file_line, ngras)| {
                ngras.into_iter().for_each(|ngram| {
                    self.ngram_to_file_line
                        .entry(ngram)
                        .or_default()
                        .push(file_line.clone());
                });
            });
        Ok(())
    }
}

impl FileIDBuilder {
    fn get_or_insert(&mut self, path: &AbsPath) -> &FileIndex {
        let new_id = self.file_to_id.len() as u32;
        self.file_to_id
            .entry(path.clone())
            .or_insert_with(|| FileIndex { file_id: new_id })
    }
}
#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf};

    use super::*;

    #[test]
    fn test_index() {
        let mut index_builder = IndexBuilder::new("test".to_string()).unwrap();
        index_builder
            .index("test/file".to_string(), 3_usize)
            .unwrap();
        assert_eq!(
            index_builder
                .index
                .ngram_to_file_line
                .get(&NgramIndex::new(&[51, 52, 53]))
                .unwrap(),
            &vec![FileLine {
                file_id: FileIndex { file_id: 0 },
                line_id: LineIndex { line: 1 }
            }]
        );
    }
    #[test]
    fn test_ngram_index_from_str() {
        let ngrams = NgramIndex::from_str("hello", 3);
        assert_eq!(ngrams.len(), 3);
        assert_eq!(
            ngrams[0],
            NgramIndex::Char(Box::new((
                b'h',
                NgramIndex::Char(Box::new((b'e', NgramIndex::Last(b'l'))))
            )))
        );
        assert_eq!(
            ngrams[1],
            NgramIndex::Char(Box::new((
                b'e',
                NgramIndex::Char(Box::new((b'l', NgramIndex::Last(b'l'))))
            )))
        );
        assert_eq!(
            ngrams[2],
            NgramIndex::Char(Box::new((
                b'l',
                NgramIndex::Char(Box::new((b'l', NgramIndex::Last(b'o'))))
            )))
        );
    }

    #[test]
    #[should_panic(expected = "Ngram cannot be empty")]
    fn test_ngram_index_new_empty() {
        NgramIndex::new(&[]);
    }

    #[test]
    fn test_ngram_index_new() {
        let ngram = NgramIndex::new(b"abc");
        assert_eq!(
            ngram,
            NgramIndex::Char(Box::new((
                b'a',
                NgramIndex::Char(Box::new((b'b', NgramIndex::Last(b'c'))))
            )))
        );
    }

    #[test]
    fn test_file_content_new() {
        let file_content = FileContent::new("test/textfile".to_string());
        assert!(file_content.is_ok());
        let content = file_content.unwrap();
        assert!(!content.lines().is_empty());
        assert!(content.get_line(0).is_none());
        assert!(content.get_line(1).is_some());
    }

    #[test]
    fn test_file_line() {
        let file_content = FileContent::new("test/file".to_string());
        assert!(file_content.is_ok());
        let content = file_content.unwrap();
        assert_eq!(content.get_line(2).unwrap(), "98765");
    }

    #[test]
    fn test_abs_path_new() {
        let abs_path = AbsPath::new("test/../test/textfile".to_string());
        let path = abs_path.unwrap();
        assert!(PathBuf::from(path.path()).is_absolute());
    }

    #[test]
    fn test_index_builder_new() {
        let index_builder = IndexBuilder::new("test/textfile".to_string());
        assert!(index_builder.is_ok());
        let builder = index_builder.unwrap();
        let abs_path = env::current_dir().unwrap().join("test/textfile");
        assert_eq!(builder.path.path(), abs_path.to_str().unwrap());
    }
    #[test]
    fn test_file_id_builder_make_or_insert() {
        let mut file_id_builder = FileIDBuilder::default();
        let abs_path = AbsPath::new("test/textfile".to_string()).unwrap();
        let file_index = file_id_builder.get_or_insert(&abs_path);
        assert_eq!(file_index.file_id, 0);
        let file_index2 = file_id_builder.get_or_insert(&abs_path);
        assert_eq!(file_index2.file_id, 0);
        assert_eq!(file_id_builder.file_to_id.len(), 1);
    }
}
