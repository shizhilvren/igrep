
// #[derive(Debug)]
// pub enum NgramTreeResult {
//     ALL,
//     Set(HashSet<FileLineIndex>),
// }

// impl NgramTreeResult {
//     pub fn alternation(self: Self, b: Self) -> Self {
//         match (self, b) {
//             (Self::Set(sub1), Self::Set(sub2)) => {
//                 Self::Set(sub1.union(&sub2).cloned().collect::<HashSet<_>>())
//             }
//             (_, _) => Self::ALL,
//         }
//     }
//     pub fn concat(self: Self, b: Self) -> Self {
//         match (self, b) {
//             (Self::Set(sub1), Self::Set(sub2)) => {
//                 Self::Set(sub1.intersection(&sub2).cloned().collect::<HashSet<_>>())
//             }
//             (Self::Set(sub1), Self::ALL) => Self::Set(sub1),
//             (Self::ALL, Self::Set(sub2)) => Self::Set(sub2),
//             (Self::ALL, Self::ALL) => Self::ALL,
//         }
//     }
// }

// impl NgramTree {



//     pub fn get_file_lines(
//         &self,
//         ngram_to_data: &HashMap<NgramIndex, &NgramData>,
//     ) -> NgramTreeResult {
//         let ans = match self {
//             Self::ALL => NgramTreeResult::ALL,
//             Self::Empty => NgramTreeResult::Set(HashSet::new()),
//             Self::Gram(e) => match ngram_to_data.get(e) {
//                 Some(data) => NgramTreeResult::Set(
//                     data.file_lines()
//                         .clone()
//                         .into_iter()
//                         .collect::<HashSet<_>>(),
//                 ),
//                 _ => NgramTreeResult::Set(HashSet::new()),
//             },
//             Self::Alternation(sub) => sub
//                 .iter()
//                 .map(|t| t.get_file_lines(ngram_to_data))
//                 .fold(NgramTreeResult::Set(HashSet::new()), |ans, result| {
//                     ans.alternation(result)
//                 }),

//             Self::Concat(sub) => sub
//                 .iter()
//                 .map(|t| t.get_file_lines(ngram_to_data))
//                 .fold(NgramTreeResult::ALL, |ans, result| {
//                     ans.concat(result)
//                 }),
//         };
//         ans
//     }
// }

// impl Engine {


//     pub fn ngram(&self, n: u8) -> NgramTree {
//         Self::ngram_from_hir(&self.regex, n)
//     }


// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_ngram_tree_basic_types() -> Result<(), Box<dyn std::error::Error>> {
//         let engine = Engine::new("ab.*cd")?;
//         let a = engine.ngram(3);
//         assert!(a.is_all());
//         Ok(())
//     }

//     #[test]
//     fn test_1() -> Result<(), Box<dyn std::error::Error>> {
//         let engine = Engine::new("abcd.*cwerfasdfd")?;
//         let a = engine.ngram(3);
//         assert!(!a.is_all());
//         Ok(())
//     }

//     #[test]
//     fn test_2() -> Result<(), Box<dyn std::error::Error>> {
//         let engine = Engine::new("abcd.*cwer|fasdfd")?;
//         let a = engine.ngram(3);
//         assert!(!a.is_all());
//         Ok(())
//     }
//     #[test]
//     fn test_3() -> Result<(), Box<dyn std::error::Error>> {
//         let engine = Engine::new("[abcd.*cwer|fasdfd]{1,2}")?;
//         let a = engine.ngram(3);
//         assert!(!a.is_all());
//         Ok(())
//     }
// }
