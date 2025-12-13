use crate::string_utils::{escape_string, unescape_string};
use pest::iterators::Pair as PestPair;
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};

/// 解析器
#[derive(Parser)]
#[grammar = "localisation/localisation.pest"]
struct LocParser;

/// 本地化文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub header: Header,
    pub items: Vec<Item>,
}

/// 文件头
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub lang: String,
}

/// 条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    Pair(Pair),
    Comment(String),
}

/// 键值对
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pair {
    pub key: String,
    pub version: Option<u32>,
    pub value: String,
}

/// 根据字符串解析 AST
///
/// # Arguments
///
/// * `input`: 文件内容
///
/// returns: Result<File, String>
///
/// # Examples
///
/// ```
///
/// ```
pub fn parse_str(input: &str) -> Result<File, String> {
    let pairs = LocParser::parse(Rule::file, input).map_err(|e| e.to_string())?;
    let file = pairs.into_iter().next().unwrap();
    parse_file(file)
}

/// 根据本地化文件解析内容
///
/// # Arguments
///
/// * `p`: 本地化文件
///
/// returns: Result<File, String>
///
/// # Examples
///
/// ```
///
/// ```
fn parse_file(p: PestPair<Rule>) -> Result<File, String> {
    let mut header = None;
    let mut items = Vec::new();

    for child in p.into_inner() {
        match child.as_rule() {
            Rule::header => header = Some(parse_header(child)),
            Rule::item => {
                if let Some(it) = parse_item(child) {
                    items.push(it);
                }
            }
            _ => {}
        }
    }

    Ok(File {
        header: header.ok_or("解析失败！文件头丢失")?,
        items,
    })
}

/// 根据文件头解析语种标识
///
/// # Arguments
///
/// * `p`: 文件头
///
/// returns: Header
///
/// # Examples
///
/// ```
///
/// ```
fn parse_header(p: PestPair<Rule>) -> Header {
    let mut inner = p.into_inner();
    let lang = inner.next().unwrap().as_str().to_string();
    Header { lang }
}

/// 根据条目列表解析条目
///
/// # Arguments
///
/// * `p`: 条目列表
///
/// returns: Option<Item>
///
/// # Examples
///
/// ```
///
/// ```
fn parse_item(p: PestPair<Rule>) -> Option<Item> {
    let inner = p.into_inner().next()?;

    match inner.as_rule() {
        Rule::pair => Some(Item::Pair(parse_pair(inner))),
        Rule::comment => Some(Item::Comment(inner.as_str().to_string())),
        _ => None,
    }
}

/// 根据条目解析键值对
///
/// # Arguments
///
/// * `p`: 条目
///
/// returns: Pair
///
/// # Examples
///
/// ```
///
/// ```
fn parse_pair(p: PestPair<Rule>) -> Pair {
    let mut key = String::new();
    let mut version = None;
    let mut value = String::new();

    for part in p.into_inner() {
        match part.as_rule() {
            Rule::key => key = part.as_str().to_string(),
            Rule::version => version = Some(part.as_str().parse::<u32>().unwrap()),
            Rule::inner => value = unescape_string(part.as_str()),
            _other => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "【parse_pair】遗漏规则：{:?}，文本：{:?}",
                    _other,
                    part.as_str()
                );
            }
        }
    }

    Pair {
        key,
        version,
        value,
    }
}

/// 序列化 AST 为字符串
///
/// # Arguments
///
/// * `ast`: AST
///
/// returns: String
///
/// # Examples
///
/// ```
///
/// ```
pub fn serialize_ast(ast: &File) -> String {
    let mut out = String::new();

    out.push('\u{FEFF}');
    out.push_str(&ast.header.lang);
    out.push_str(":\n");

    for it in &ast.items {
        out.push_str(&serialize_item(it));
    }

    out
}

/// 序列化条目
///
/// # Arguments
///
/// * `item`: 条目
///
/// returns: String
///
/// # Examples
///
/// ```
///
/// ```
fn serialize_item(item: &Item) -> String {
    match item {
        Item::Pair(p) => serialize_pair(p),
        Item::Comment(c) => {
            if c.starts_with('#') {
                format!(" {c}\n")
            } else {
                format!(" # {c}\n")
            }
        }
    }
}

/// 序列化键值对
///
/// # Arguments
///
/// * `pair`: 键值对
///
/// returns: String
///
/// # Examples
///
/// ```
///
/// ```
fn serialize_pair(pair: &Pair) -> String {
    // TODO: 添加配置项，通过自定义的参数控制版本号字段的处理方案：当版本号不存在时默认为0或不处理
    let ver_str = match pair.version {
        Some(v) => v.to_string(),
        None => String::new(),
    };

    let value = normalize_localisation_value(&pair.value);

    if ver_str.is_empty() {
        format!(" {}:{} {}\n", pair.key, "", value).replace(": ", ": ")
    } else {
        format!(" {}:{} {}\n", pair.key, ver_str, value)
    }
}

/// 规范化本地化词条值
///
/// # Arguments
///
/// * `value`: 值
///
/// returns: String
///
/// # Examples
///
/// ```
///
/// ```
fn normalize_localisation_value(value: &str) -> String {
    let escaped = escape_string(value);
    format!("\"{}\"", escaped)
}
