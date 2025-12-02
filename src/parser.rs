use pest::iterators::{Pair as PestPair, Pairs};
use pest::Parser;
use pest_derive::Parser;

/// 派生解析器
#[derive(Parser)]
#[grammar = "hoi4.pest"]
struct HoiParser;

/// 条目可以是键值对、值或注释
#[derive(Debug, Clone)]
pub enum Item {
    Pair(Pair),
    Value(Value),
    Comment(String),
}

/// 数组条目可以是值或注释（与 Item 唯一的不同就是少了 Pair）
#[derive(Debug, Clone)]
pub enum ArrayItem {
    Value(Value),
    Comment(String),
}

/// 键值对：key <op> value（赋值与比较）
#[derive(Debug, Clone)]
pub struct Pair {
    pub key: Key,
    pub op: Operator,
    pub value: Value,
}

/// 键：标识符或数字或日期（在历史文件中常见）
#[derive(Debug, Clone)]
pub enum Key {
    Identifier(String),
    Number(f64),
    Date(Date),
}

/// 运算符：赋值与比较
#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Le,
    Ge,
    Lt,
    Gt,
}

/// 值
#[derive(Debug, Clone)]
pub enum Value {
    Block(Block),
    Array(Array),
    Date(Date),
    Number(f64),
    Boolean(bool),
    String(String),
    Identifier(String),
}

/// 块：包含一系列条目，可嵌套，用于复杂结构（例如触发器）
#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<Item>,
}

/// 数组：包含一系列值（用于无键值对的块）
#[derive(Debug, Clone)]
pub struct Array {
    pub values: Vec<ArrayItem>,
}

/// 日期（YYYY.MM.DD(.HH)）
#[derive(Debug, Clone)]
pub struct Date {
    pub y: u32,
    pub m: u8,
    pub d: u8,
    pub h: Option<u8>,
}

/// 从字符串解析 AST
pub fn parse_str(input: &str) -> Result<Vec<Item>, String> {
    let pairs = HoiParser::parse(Rule::file, input).map_err(|e| e.to_string())?;
    Ok(parse_file(pairs))
}

/// 序列化 AST 为字符串
pub fn serialize_ast(items: &[Item]) -> String {
    let mut out = String::new();
    for it in items {
        out.push_str(&serialize_item(it, 0));
    }
    out
}

/// 将日期字符串解析为 Date 结构体
fn parse_date_str(s: &str) -> Date {
    let mut parts = s.split('.');
    let y = parts.next().unwrap().parse::<u32>().unwrap();
    let m = parts.next().unwrap().parse::<u8>().unwrap();
    let d = parts.next().unwrap().parse::<u8>().unwrap();
    let h = parts.next().map(|x| x.parse::<u8>().unwrap());
    Date { y, m, d, h }
}

/// 解析键
fn parse_key(p: pest::iterators::Pair<Rule>) -> Key {
    match p.as_rule() {
        Rule::identifier => Key::Identifier(p.as_str().to_string()),
        Rule::number => Key::Number(p.as_str().parse::<f64>().unwrap()),
        Rule::date => Key::Date(parse_date_str(p.as_str())),
        _ => Key::Identifier(p.as_str().to_string()),
    }
}

/// 解析运算符
fn parse_operator(p: PestPair<Rule>) -> Operator {
    match p.as_str() {
        "=" => Operator::Eq,
        "<=" => Operator::Le,
        ">=" => Operator::Ge,
        "<" => Operator::Lt,
        ">" => Operator::Gt,
        _ => Operator::Eq,
    }
}

/// 解析值
fn parse_value(p: pest::iterators::Pair<Rule>) -> Value {
    match p.as_rule() {
        Rule::string => {
            let inner = p.into_inner().next().unwrap();
            let s = inner.as_str();
            Value::String(s.to_string())
        }
        Rule::identifier => Value::Identifier(p.as_str().to_string()),
        Rule::number => Value::Number(p.as_str().parse::<f64>().unwrap()),
        Rule::date => Value::Date(parse_date_str(p.as_str())),
        Rule::boolean => Value::Boolean(p.as_str() == "yes"),
        _ => Value::Identifier(p.as_str().to_string()),
    }
}

/// 解析块内容
///
/// 判断块是纯数组（Array）还是混合键值对的块（Block）
/// 如果块中只包含值或注释，则解析为 Array，否则解析为 Block
fn parse_block(p: pest::iterators::Pair<Rule>) -> Value {
    let mut items: Vec<Item> = Vec::new();
    let mut only_values = true;

    for child in p.into_inner() {
        let parsed = parse_item(child.clone());
        match &parsed {
            Item::Value(_) => {
                items.push(parsed);
            }
            Item::Pair(_) => {
                // 一旦出现键值对，就不能是数组
                only_values = false;
                items.push(parsed);
            }
            Item::Comment(_) => {
                items.push(parsed);
            }
        }
    }

    if only_values {
        // 转换 Vec<Item> 为 Vec<ArrayItem>
        let array_items: Vec<ArrayItem> = items
            .into_iter()
            .map(|item| match item {
                Item::Value(v) => ArrayItem::Value(v),
                Item::Comment(c) => ArrayItem::Comment(c),
                _ => unreachable!("Should not happen if only_atoms is true"),
            })
            .collect();

        Value::Array(Array {
            values: array_items,
        })
    } else {
        Value::Block(Block { items })
    }
}

/// 递归解析 Item
///
/// Item 可以是键值对 (Pair)、值 (Value) 或注释 (Comment)
fn parse_item(p: pest::iterators::Pair<Rule>) -> Item {
    match p.as_rule() {
        Rule::item => {
            let mut inner = p.into_inner();
            if let Some(child) = inner.next() {
                return parse_item(child);
            }

            // 空 item 默认为空标识符（理论上不应发生）
            Item::Value(Value::Identifier(String::new()))
        }
        Rule::pair => {
            let mut it = p.into_inner();
            let key = parse_key(it.next().unwrap());
            let op = parse_operator(it.next().unwrap());
            let val_pair = it.next().unwrap();

            let value = match val_pair.as_rule() {
                Rule::value => {
                    let mut inner = val_pair.into_inner();
                    let v = inner.next().unwrap();
                    match v.as_rule() {
                        Rule::block => parse_block(v),
                        Rule::string
                        | Rule::date
                        | Rule::number
                        | Rule::boolean
                        | Rule::identifier => parse_value(v),
                        _ => Value::Identifier(v.as_str().to_string()),
                    }
                }
                Rule::block => parse_block(val_pair),
                Rule::string | Rule::date | Rule::number | Rule::boolean | Rule::identifier => {
                    parse_value(val_pair)
                }
                _ => Value::Identifier(val_pair.as_str().to_string()),
            };

            Item::Pair(Pair { key, op, value })
        }
        Rule::value => {
            let mut inner = p.into_inner();
            let v = inner.next().unwrap();
            let val = match v.as_rule() {
                Rule::block => parse_block(v),
                Rule::string | Rule::date | Rule::number | Rule::boolean | Rule::identifier => {
                    parse_value(v)
                }
                _ => Value::Identifier(v.as_str().to_string()),
            };

            Item::Value(val)
        }
        Rule::comment => Item::Comment(p.as_str().to_string()),
        _ => Item::Value(Value::Identifier(p.as_str().to_string())),
    }
}

/// 解析整个文件
///
/// 将 Pest 解析结果转换为 Item 列表
fn parse_file(pairs: Pairs<Rule>) -> Vec<Item> {
    let mut items = Vec::new();
    let file = pairs.into_iter().next().unwrap();
    for child in file.into_inner() {
        if child.as_rule() == Rule::EOI {
            continue;
        }
        items.push(parse_item(child));
    }
    items
}

/// 序列化日期
///
/// 输出格式为 YYYY.MM.DD 或 "YYYY.MM.DD.HH"
fn serialize_date(d: &Date) -> String {
    match d.h {
        Some(h) => format!("\"{}.{}.{}.{}\"", d.y, d.m, d.d, h),
        None => format!("{}.{}.{}", d.y, d.m, d.d),
    }
}

/// 序列化键
fn serialize_key(k: &Key) -> String {
    match k {
        Key::Identifier(s) => s.clone(),
        Key::Number(n) => n.to_string(),
        Key::Date(d) => serialize_date(d),
    }
}

/// 序列化值
///
/// 处理各种 Value 类型的字符串表示，包括缩进和换行
fn serialize_value(v: &Value, indent: usize) -> String {
    match v {
        Value::String(s) => format!("\"{}\"", s),
        Value::Identifier(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Date(d) => serialize_date(d),
        Value::Boolean(b) => {
            if *b {
                "yes".to_string()
            } else {
                "no".to_string()
            }
        }
        Value::Array(arr) => {
            // 预渲染数组元素
            let rendered: Vec<String> = arr
                .values
                .iter()
                .map(|item| match item {
                    ArrayItem::Value(v) => serialize_value(v, 0),
                    ArrayItem::Comment(c) => c.clone(),
                })
                .collect();

            let mut out = String::new();
            out.push_str("{\n");

            let mut line = String::new();
            for (idx, elem) in rendered.iter().enumerate() {
                let is_comment = matches!(arr.values[idx], ArrayItem::Comment(_));

                // 注释独占一行
                if is_comment {
                    if !line.is_empty() {
                        out.push_str(&"\t".repeat(indent + 1));
                        out.push_str(&line);
                        out.push('\n');
                        line.clear();
                    }
                    out.push_str(&"\t".repeat(indent + 1));
                    out.push_str(elem);
                    out.push('\n');
                    continue;
                }

                // 简单的自动换行逻辑：若行长度超过 120 则换行
                let sep = if line.is_empty() { "" } else { " " };
                let prospective_len = line.len() + sep.len() + elem.len();

                if !line.is_empty() && prospective_len > 120 {
                    out.push_str(&"\t".repeat(indent + 1));
                    out.push_str(&line);
                    out.push('\n');
                    line.clear();
                }

                if line.is_empty() {
                    line.push_str(elem);
                } else {
                    line.push(' ');
                    line.push_str(elem);
                }

                // 处理最后一个元素
                if idx == rendered.len() - 1 {
                    out.push_str(&"\t".repeat(indent + 1));
                    out.push_str(&line);
                    out.push('\n');
                }
            }

            out.push_str(&"\t".repeat(indent));
            out.push_str("}\n");
            out
        }
        Value::Block(block) => {
            let mut out = String::new();
            out.push_str("{\n");
            for it in &block.items {
                out.push_str(&serialize_item(it, indent + 1));
            }
            out.push_str(&"\t".repeat(indent));
            out.push_str("}\n");
            out
        }
    }
}

/// 序列化条目
///
/// 负责将 Item (Pair/Value/Comment) 转换为格式化的字符串
fn serialize_item(i: &Item, indent: usize) -> String {
    match i {
        Item::Pair(pair) => {
            let mut line = String::new();
            line.push_str(&"\t".repeat(indent));
            line.push_str(&serialize_key(&pair.key));
            line.push_str(" ");
            line.push_str(match pair.op {
                Operator::Eq => "=",
                Operator::Le => "<=",
                Operator::Ge => ">=",
                Operator::Lt => "<",
                Operator::Gt => ">",
            });
            line.push_str(" ");
            match pair.value {
                // 块和数组自带换行和缩进逻辑，无需额外处理
                Value::Array(_) | Value::Block(_) => {
                    line.push_str(&serialize_value(&pair.value, indent));
                }
                _ => {
                    line.push_str(&serialize_value(&pair.value, indent));
                    line.push('\n');
                }
            }
            line
        }
        Item::Value(v) => {
            let mut line = String::new();
            line.push_str(&"\t".repeat(indent));
            match v {
                Value::Array(_) | Value::Block(_) => {
                    line.push_str(&serialize_value(v, indent));
                }
                _ => {
                    line.push_str(&serialize_value(v, indent));
                    line.push('\n');
                }
            }
            line
        }
        Item::Comment(s) => {
            let mut line = String::new();
            line.push_str(&"\t".repeat(indent));
            line.push_str(s);
            line.push('\n');
            line
        }
    }
}
