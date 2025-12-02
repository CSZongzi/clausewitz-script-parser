#[cfg(test)]
mod hoi4_tests {
    use clausewitz_script_parser::parser::{parse_str, serialize_ast};
    use std::fs;
    use std::path::Path;
    /// 测试 common/characters 文件
    #[test]
    fn test_characters() {
        const BASE_INPUT_PATH: &str = "res/hoi4/common/characters/TST.txt";
        const OUTPUT_DIR: &str = "output/hoi4/common/characters";
        const OUTPUT_AST_PATH: &str = "output/hoi4/common/characters/TST.ast";
        const OUTPUT_TXT_PATH: &str = "output/hoi4/common/characters/TST.txt";

        let input_path = Path::new(BASE_INPUT_PATH);
        let input = fs::read_to_string(input_path).expect("读取失败！文件不存在");
        let ast = parse_str(&input).expect("解析失败！内容不合法");

        fs::create_dir_all(OUTPUT_DIR).expect("创建失败！路径不合法");

        fs::write(OUTPUT_AST_PATH, format!("{:#?}", ast)).expect("写入失败！抽象语法树异常");
        fs::write(OUTPUT_TXT_PATH, serialize_ast(&ast)).expect("写入失败！文本异常");

        let output = fs::read_to_string(OUTPUT_TXT_PATH).expect("读取失败！生成文本文件不存在");
        let output_ast = parse_str(&output).expect("解析失败！内容不合法");

        // TODO: 这里的逻辑有问题，考虑如何判断生成的文件是否与源文件一致，可能需要给出预定义的文件

        assert_eq!(
            format!("{:#?}", ast),
            format!("{:#?}", output_ast),
            "源文件和生成文件内容不一致"
        );
    }
}
