#[cfg(test)]
mod hoi4_tests {
    use clausewitz_script_parser::localisation::localisation::{
        parse_str as parse_loc_str, serialize_ast as serialize_loc_ast,
    };
    use clausewitz_script_parser::script::script::{
        parse_str as parse_scr_parse, serialize_ast as serialize_scr_ast,
    };
    use std::fmt::{Debug, Display};
    use std::fs;
    use std::path::Path;

    /// # 测试函数
    ///
    /// # Arguments
    ///
    /// * `base_input_path`: 源 HOI4 脚本文件
    /// * `expected_txt_path`: 期望输出文件
    /// * `output_dir`: 输出目录
    /// * `output_ast_path`: AST 输出文件路径
    /// * `output_txt_path`: 生成文本输出文件路径
    /// * `parse_fn`: 解析函数
    /// * `serialize_fn`: 序列化函数
    ///
    /// returns: ()
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    fn run_test_with<Ast, ParseErr, ParseFn, SerFn>(
        base_input_path: &str,
        expected_txt_path: &str,
        output_dir: &str,
        output_ast_path: &str,
        output_txt_path: &str,
        parse_fn: ParseFn,
        serialize_fn: SerFn,
    ) where
        Ast: Debug,
        ParseErr: Display + Debug,
        ParseFn: Fn(&str) -> Result<Ast, ParseErr>,
        SerFn: Fn(&Ast) -> String,
    {
        use std::fs;
        use std::path::Path;

        /// # 统一换行符
        ///
        /// # Arguments
        ///
        /// * `s`: 输入字符串
        ///
        /// returns: String
        ///
        /// # Examples
        ///
        /// ```
        ///
        /// ```
        fn normalize_newlines(s: &str) -> String {
            s.replace("\r\n", "\n").replace('\r', "\n")
        }

        let input_path = Path::new(base_input_path);
        let input = fs::read_to_string(input_path).expect("读取失败！源文件不存在");
        let ast = parse_fn(&input).unwrap_or_else(|e| panic!("{}", e));

        fs::create_dir_all(output_dir).expect("创建失败！输出路径不合法");

        fs::write(output_ast_path, format!("{:#?}", ast)).expect("写入失败！抽象语法树异常");
        fs::write(output_txt_path, serialize_fn(&ast)).expect("写入失败！文本输出异常");

        let exp_txt = fs::read_to_string(expected_txt_path).expect("读取失败！期望输出文件不存在");
        let act_txt = fs::read_to_string(output_txt_path).expect("读取失败！生成文本文件不存在");

        let exp_norm = normalize_newlines(&exp_txt);
        let act_norm = normalize_newlines(&act_txt);

        assert_eq!(exp_norm, act_norm, "生成文件与期望输出不一致");
    }

    /// 测试 common/characters 文件
    #[test]
    fn test_characters() {
        const BASE_INPUT_PATH: &str = "res/hoi4/common/characters/TST.txt";
        const EXPECTED_TXT_PATH: &str = "res/hoi4/common/characters/TST_expected.txt";
        const OUTPUT_DIR: &str = "output/hoi4/common/characters";
        const OUTPUT_AST_PATH: &str = "output/hoi4/common/characters/TST.ast";
        const OUTPUT_TXT_PATH: &str = "output/hoi4/common/characters/TST.txt";

        run_test_with(
            BASE_INPUT_PATH,
            EXPECTED_TXT_PATH,
            OUTPUT_DIR,
            OUTPUT_AST_PATH,
            OUTPUT_TXT_PATH,
            |s| parse_scr_parse(s),
            |ast| serialize_scr_ast(ast),
        );
    }

    /// 测试 common/ideas 文件
    #[test]
    fn test_ideas() {
        const BASE_INPUT_PATH: &str = "res/hoi4/common/ideas/test.txt";
        const EXPECTED_TXT_PATH: &str = "res/hoi4/common/ideas/test.expected.txt";
        const OUTPUT_DIR: &str = "output/hoi4/common/ideas";
        const OUTPUT_AST_PATH: &str = "output/hoi4/common/ideas/test.ast";
        const OUTPUT_TXT_PATH: &str = "output/hoi4/common/ideas/test.txt";

        run_test_with(
            BASE_INPUT_PATH,
            EXPECTED_TXT_PATH,
            OUTPUT_DIR,
            OUTPUT_AST_PATH,
            OUTPUT_TXT_PATH,
            |s| parse_scr_parse(s),
            |ast| serialize_scr_ast(ast),
        );
    }

    /// 测试 res/hoi4/localisation
    #[test]
    fn test_localisation() {
        const BASE_INPUT_PATH: &str = "res/hoi4/localisation/test_l_simp_chinese.yml";
        const EXPECTED_TXT_PATH: &str = "res/hoi4/localisation/test_l_simp_chinese.expected.yml";
        const OUTPUT_DIR: &str = "output/hoi4/localisation";
        const OUTPUT_AST_PATH: &str = "output/hoi4/localisation/test_l_simp_chinese.ast";
        const OUTPUT_TXT_PATH: &str = "output/hoi4/localisation/test_l_simp_chinese.yml";

        run_test_with(
            BASE_INPUT_PATH,
            EXPECTED_TXT_PATH,
            OUTPUT_DIR,
            OUTPUT_AST_PATH,
            OUTPUT_TXT_PATH,
            |s| parse_loc_str(s),
            |ast| serialize_loc_ast(ast),
        );
    }
}
