mod script;

use crate::script::script::{parse_str, serialize_ast};
use std::fs;
use std::path::Path;

const HOI4_COMMON_DIR: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\Hearts of Iron IV\common";

fn main() {
    walk_directory(HOI4_COMMON_DIR).expect("处理目录时出错");
}

fn walk_directory<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let entries = fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk_directory(&path)?;
        } else if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
            process_file(&path)?;
        }
    }

    Ok(())
}

fn process_file<P: AsRef<Path>>(file_path: P) -> std::io::Result<()> {
    let file_path = file_path.as_ref();

    let input = fs::read_to_string(file_path)?;

    match parse_str(&input) {
        Ok(ast) => {
            let relative_path = file_path.strip_prefix(HOI4_COMMON_DIR).unwrap_or(file_path);
            let output_path = Path::new("output").join(relative_path);

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let ast_output_path = output_path.with_extension("ast");
            fs::write(&ast_output_path, format!("{:#?}", ast))
                .expect(&format!("写入 AST 文件失败: {:?}", ast_output_path));

            let txt_output_path = output_path;
            fs::write(&txt_output_path, serialize_ast(&ast))
                .expect(&format!("写入文本文件失败: {:?}", txt_output_path));
        }
        Err(e) => {
            eprintln!("解析文件 {:?} 时出错: {}", file_path, e);
        }
    }

    Ok(())
}
