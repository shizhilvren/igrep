pub mod config;
pub mod index_builder;
pub mod index_file;
pub mod index_regex;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_with_config() {
        let cfg = config::Config::new(
            "path/to/config".to_string(),
        );
        assert_eq!(cfg.path(), "path/to/config");
    }
}
