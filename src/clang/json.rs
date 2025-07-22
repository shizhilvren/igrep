use crate::clang::index::{FileLocation, FunctionResult, IndexResult, OneFileLocation};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::fs;
pub struct FileJson {
    pub path: String,
    pub content: Vec<LineJson>,
}

pub struct LineJson {
    pub files: Vec<Token>,
}

pub struct Token {
    token: String,
    classes: Vec<Class>,
    id: Option<Id>,
    url: Option<String>,
}

pub struct Class(String);
pub struct Id(String);

fn makeTokens(index: &IndexResult) -> Result<HashMap<String, Vec<&OneFileLocation>>> {
    let mut locs = index
        .get_functions()
        .iter()
        .map(|(usr, fr)| {
            fr.calls()
                .iter()
                .chain(fr.definitions().iter())
                .chain(fr.declarations().iter())
        })
        .flatten()
        .map(|loc| (loc.file(), loc.loc()))
        .fold(HashMap::new(), |mut acc, (file, loc)| {
            acc.entry(file.to_string())
                .or_insert_with(Vec::new)
                .push(loc);
            acc
        })
        .into_iter()
        .map(|(key, mut val)| {
            val.sort();
            val.as_slice().windows(2).for_each(|e| {
                let a = e[0];
                let b = e[1];
                assert!(
                    a.offset() + a.len() <= b.offset(),
                    "File: {}, {} {} {} {} - {} {} {} {}",
                    key,
                    a.line(),
                    a.column(),
                    a.offset(),
                    a.len(),
                    b.line(),
                    b.column(),
                    b.offset(),
                    b.len()
                );
            });
            (key, val)
        })
        .collect::<HashMap<_, _>>();
    // locs.sort();
    Ok(locs)
}

fn splitOneFile(file: String, tokens: HashMap<String, Vec<&OneFileLocation>>) -> Result<()> {
    let lines =
        fs::read_to_string(&file).map_err(|e| anyhow!("Failed to read file {}: {}", &file, e))?.lines();
    let tokens = tokens
        .get(&file)
        .ok_or_else(|| anyhow!("No tokens found for file {}", &file))?;
    let mut points = tokens.iter().fold(vec![0, lines.len() as u32], |mut acc, loc| {
        acc.push(loc.offset());
        acc.push(loc.offset() + loc.len());
        acc
    });
    points.sort();
    points.dedup();
    Ok(())
}
