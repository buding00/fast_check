use yara_x::Compiler;

pub fn new_rule<'a>() -> Compiler<'a> {
    let mut compiler = Compiler::new();
    crate::Asset::iter().for_each(|path| {
        let file_content = crate::Asset::get(path.as_ref()).unwrap();
        let rules_string = std::str::from_utf8(file_content.data.as_ref()).unwrap();
        if let Err(err) = compiler.add_source(rules_string) {
            // 处理编译错误，例如打印错误信息
            eprintln!("Error adding rules from {}: {:?}", path, err);
        }
    });
    return compiler;
}

mod test {
    #[test]
    pub fn test_new_rule() {
        use super::*;
        let compiler = new_rule();
        print!("{}", compiler.build().iter().len());
    }
    #[test]
    fn yara_test() {
        // Create a compiler.
        let mut compiler = yara_x::Compiler::new();

        // Add some YARA source code to compile.
        compiler
            .add_source(
                r#"
    rule lorem_ipsum {
      strings:
        $ = "Lorem ipsum"
      condition:
        all of them
    }
"#,
            )
            .unwrap();

        // Obtain the compiled YARA rules.
        let rules = compiler.build();

        // Create a scanner that uses the compiled rules.
        let mut scanner = yara_x::Scanner::new(&rules);

        // Scan some data.
        let results = scanner.scan("Lorem ipsum".as_bytes()).unwrap();
        results
            .matching_rules()
            .for_each(|rule| println!("{}", rule.identifier()));
        assert_eq!(results.matching_rules().len(), 1);
    }
}
