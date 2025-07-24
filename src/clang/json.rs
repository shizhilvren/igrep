use crate::clang::index::{FileLocation, FunctionResult, IndexResult, OneFileLocation};
use anyhow::{Result, anyhow};
use clang::Usr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;


pub struct SymbolsJson{
    pub symbols: Vec<SymbolJson>,
}

#[derive(Serialize, Deserialize)]
pub struct SymbolJson {
    pub name: String,
    pub definition: Vec<FileLineLink>,
    pub declaration: Vec<FileLineLink>,
    pub call: Vec<FileLineLink>,
}

#[derive(Serialize, Deserialize)]
pub struct FileLineLink {
    pub file: String,
    pub line: u32,
}

#[derive(Serialize, Deserialize)]
pub struct FileJson {
    pub path: String,
    pub content: Content,
}
#[derive(Serialize, Deserialize)]
pub struct Content {
    pub content: Vec<LineJson>,
}
#[derive(Serialize, Deserialize)]
pub struct LineJson {
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize)]
pub struct Token {
    token: String,
    classes: Vec<Class>,
    id: Option<Id>,
    url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Class(String);
#[derive(Serialize, Deserialize)]
pub struct Id(String);

enum OneFileLocationData {
    FunctionDecl(Usr),
    FunctionDef(FunctionDef),
    FunctionCall(Usr),
}
struct FunctionDef {
    usr: Usr,
    only_one: bool,
}

fn make_tokens(index: &IndexResult) -> Result<HashMap<String, Vec<(&OneFileLocation, &Usr)>>> {
    let locs = index
        .get_functions()
        .iter()
        .map(|(usr, fr)| {
            let len = fr.calls().len() + fr.definitions().len() + fr.declarations().len();
            fr.calls()
                .iter()
                .chain(fr.definitions().iter())
                .chain(fr.declarations().iter())
                .zip(std::iter::repeat(usr))
        })
        .flatten()
        .map(|(loc, usr)| (loc.file(), loc.loc(), usr))
        .fold(HashMap::new(), |mut acc, (file, loc, usr)| {
            acc.entry(file.to_string())
                .or_insert_with(Vec::new)
                .push((loc, usr));
            acc
        })
        .into_iter()
        .map(|(key, mut val)| {
            val.sort();
            val.dedup();
            val.as_slice().windows(2).for_each(|e| {
                let a = e[0];
                let b = e[1];
                // assert!(
                //     a.offset() + a.len() <= b.offset(),
                //     "File: {}, {} {} {} {} - {} {} {} {}",
                //     key,
                //     a.line(),
                //     a.column(),
                //     a.offset(),
                //     a.len(),
                //     b.line(),
                //     b.column(),
                //     b.offset(),
                //     b.len()
                // );
            });
            (key, val)
        })
        .collect::<HashMap<_, _>>();
    // locs.sort();
    Ok(locs)
}

fn split_one_file(
    file: String,
    tokens: HashMap<String, Vec<(&OneFileLocation, &Usr)>>,
) -> Result<FileJson> {
    let lines = fs::read_to_string(&file)
        .map_err(|e| anyhow!("Failed to read file {}: {}", &file, e))?
        .lines()
        .into_iter()
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    let tokens = tokens
        .get(&file)
        .ok_or_else(|| anyhow!("No tokens found for file {}", &file))?;
    let acc = lines
        .iter()
        .map(|line| HashMap::from([(0, vec![]), (line.len() as u32, vec![])]))
        .collect::<Vec<_>>();
    let points = tokens
        .iter()
        .fold(acc, |mut acc, (loc, usr)| {
            let start = loc.column() - 1;
            let end = start + loc.len();
            acc[(loc.line() - 1) as usize]
                .entry(start)
                .or_insert(vec![])
                .push(usr);
            acc[(loc.line() - 1) as usize].entry(end).or_insert(vec![]);
            acc
        })
        .into_iter()
        .map(|points| {
            let mut points = points.into_iter().collect::<Vec<_>>();
            points.sort_by_key(|a| a.0);
            points
        })
        .collect::<Vec<_>>();
    let content = Content {
        content: points
            .into_iter()
            .zip(lines.into_iter())
            .map(|(points, line)| {
                let tokens = points
                    .as_slice()
                    .windows(2)
                    .map(|pair| {
                        let data = &pair[0].1;
                        let id = match data.len() {
                            1 => Some(Id(data[0].0.clone())), // No tokens in this range
                            _ => None,
                        };
                        let classes = data
                            .iter()
                            .map(|usr| Class("fn".to_string()))
                            .collect::<Vec<_>>();
                        let start = pair[0].0 as usize;
                        let end = pair[1].0 as usize;
                        let token = line[start..end].to_string();
                        Token {
                            token,
                            classes: classes,
                            id: id,
                            url: None,
                        }
                    })
                    .collect::<Vec<_>>();
                LineJson { tokens: tokens }
            })
            .collect::<Vec<_>>(),
    };
    let file_json = FileJson {
        path: file,
        content,
    };
    Ok(file_json)
}

impl FileJson {
    pub fn from_index(index: &IndexResult, file: String) -> Result<FileJson> {
        let tokens = make_tokens(index)?;
        let file_json = split_one_file(file, tokens.clone())?;
        Ok(file_json)
    }
}


impl SymbolsJson{
    pub fn from_index(index: &IndexResult) -> Result<SymbolsJson> {
        let mut symbols = Vec::new();
        for (usr, fr) in index.get_functions() {
            let name = usr.0.clone();
            let mut definition = fr
                .definitions()
                .iter()
                .map(|loc| FileLineLink {
                    file: loc.file().to_string(),
                    line: loc.loc().line(),
                })
                .collect::<Vec<_>>();
            definition.sort_by_key(|x| (x.file.clone(), x.line));
            let mut declaration = fr
                .declarations()
                .iter()
                .map(|loc| FileLineLink {
                    file: loc.file().to_string(),
                    line: loc.loc().line(),
                })
                .collect::<Vec<_>>();
            declaration.sort_by_key(|x| (x.file.clone(), x.line));
            let mut call = fr
                .calls()
                .iter()
                .map(|loc| FileLineLink {
                    file: loc.file().to_string(),
                    line: loc.loc().line(),
                })
                .collect::<Vec<_>>();
            call.sort_by_key(|x| (x.file.clone(), x.line));
            symbols.push(SymbolJson {
                name,
                definition,
                declaration,
                call,
            });
        }
        Ok(SymbolsJson { symbols })
    }
}