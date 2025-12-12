mod script;

use crate::script::script::{parse_str, serialize_ast};
use std::fs;
use std::path::Path;

fn main() {
    let input_path = Path::new("res/special.txt");
    let input = fs::read_to_string(input_path).expect("读取失败！无法读取文件");
    let ast = parse_str(&input).expect("解析失败！");
    fs::create_dir_all("output").expect("创建目录失败");
    fs::write("output/special.ast", format!("{:#?}", ast)).expect("写入 AST 失败");
    fs::write("output/special.txt", serialize_ast(&ast)).expect("写入文本失败");
}
