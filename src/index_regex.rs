


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
