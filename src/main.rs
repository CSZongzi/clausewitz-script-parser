mod parser;

use crate::parser::{parse_str, serialize_file};
use std::fs;
use std::path::Path;

fn main() {
    let input_path = Path::new("res/special.txt");
    let input = fs::read_to_string(input_path).expect("cannot read file");
    let ast = parse_str(&input).expect("unsuccessful parse");
    fs::create_dir_all("output").expect("create dir failed");
    fs::write("output/special.ast", format!("{:#?}", ast)).expect("write ast failed");
    fs::write("output/special.txt", serialize_file(&ast)).expect("write txt failed");
}
