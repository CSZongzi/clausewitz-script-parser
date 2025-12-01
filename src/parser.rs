use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

// Derive parser for Paradox script grammar
#[derive(Parser)]
#[grammar = "hoi4.pest"]
struct HoiParser;

// Operator types (assignment and comparisons)
#[derive(Debug, Clone)]
pub enum Operator {
    Eq,
    Le,
    Ge,
    Lt,
    Gt,
}

// Date type (YYYY.MM.DD(.HH))
#[derive(Debug, Clone)]
pub struct Date {
    pub y: u32,
    pub m: u8,
    pub d: u8,
    pub h: Option<u8>,
}

// Atomic values: string, identifier, number, date, boolean
#[derive(Debug, Clone)]
pub enum Atom {
    String(String),
    Ident(String),
    Number(f64),
    Date(Date),
    Bool(bool),
}

// Key types: identifier, number, date
#[derive(Debug, Clone)]
pub enum KeyAtom {
    Ident(String),
    Number(f64),
    Date(Date),
}

// Value types: atom, array (plain value list), or block (with key/values and comments)
#[derive(Debug, Clone)]
pub enum Value {
    Atom(Atom),
    Array(Vec<Atom>),
    Block(Vec<Item>),
}

// Item: key-value pair, standalone value, or comment
#[derive(Debug, Clone)]
pub enum Item {
    Pair {
        key: KeyAtom,
        op: Operator,
        value: Value,
    },
    ValueItem(Value),
    Comment(String),
}

// 公开的解析入口：从字符串解析成 AST
pub fn parse_str(input: &str) -> Result<Vec<Item>, String> {
    let pairs = HoiParser::parse(Rule::file, input).map_err(|e| e.to_string())?;
    Ok(parse_file(pairs))
}

// Serialize file
pub fn serialize_file(items: &[Item]) -> String {
    let mut out = String::new();
    for it in items {
        out.push_str(&serialize_item(it, 0));
    }
    out
}

// Parse date string into struct
fn parse_date_str(s: &str) -> Date {
    let mut parts = s.split('.');
    let y = parts.next().unwrap().parse::<u32>().unwrap();
    let m = parts.next().unwrap().parse::<u8>().unwrap();
    let d = parts.next().unwrap().parse::<u8>().unwrap();
    let h = parts.next().map(|x| x.parse::<u8>().unwrap());
    Date { y, m, d, h }
}

// Parse operator
fn parse_operator(p: Pair<Rule>) -> Operator {
    match p.as_str() {
        "=" => Operator::Eq,
        "<=" => Operator::Le,
        ">=" => Operator::Ge,
        "<" => Operator::Lt,
        ">" => Operator::Gt,
        _ => Operator::Eq,
    }
}

// Parse atomic value
fn parse_atom(p: Pair<Rule>) -> Atom {
    match p.as_rule() {
        Rule::string => {
            let inner = p.into_inner().next().unwrap();
            let s = inner.as_str();
            // Recognize date-like pattern (YYYY.MM.DD(.HH)) within quoted strings
            if let Some(d) = try_parse_date_like(s) {
                return Atom::Date(d);
            }
            Atom::String(s.to_string())
        }
        Rule::identifier => Atom::Ident(p.as_str().to_string()),
        Rule::number => Atom::Number(p.as_str().parse::<f64>().unwrap()),
        Rule::date => Atom::Date(parse_date_str(p.as_str())),
        Rule::boolean => Atom::Bool(p.as_str() == "yes"),
        _ => Atom::Ident(p.as_str().to_string()),
    }
}

// Parse key
fn parse_key(p: Pair<Rule>) -> KeyAtom {
    match p.as_rule() {
        Rule::identifier => KeyAtom::Ident(p.as_str().to_string()),
        Rule::number => KeyAtom::Number(p.as_str().parse::<f64>().unwrap()),
        Rule::date => KeyAtom::Date(parse_date_str(p.as_str())),
        _ => KeyAtom::Ident(p.as_str().to_string()),
    }
}

// Try to detect a date-like pattern from string content
fn try_parse_date_like(s: &str) -> Option<Date> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    fn all_digits(x: &str) -> bool {
        x.chars().all(|c| c.is_ascii_digit())
    }
    if !(all_digits(parts[0]) && (3..=4).contains(&parts[0].len())) {
        return None;
    }
    for i in 1..parts.len() {
        if !all_digits(parts[i]) || !(1..=2).contains(&parts[i].len()) {
            return None;
        }
    }
    let y = parts[0].parse::<u32>().ok()?;
    let m = parts[1].parse::<u8>().ok()?;
    let d = parts[2].parse::<u8>().ok()?;
    let h = if parts.len() == 4 {
        Some(parts[3].parse::<u8>().ok()?)
    } else {
        None
    };
    Some(Date { y, m, d, h })
}

// Parse block:
// - If all children are plain atomic values (no pair/comment), classify as Array(Vec<Atom>)
// - Otherwise as Block(Vec<Item>)
fn parse_block(p: Pair<Rule>) -> Value {
    let mut items: Vec<Item> = Vec::new();
    let mut atoms: Vec<Atom> = Vec::new();
    let mut only_atoms = true;
    for child in p.into_inner() {
        if child.as_rule() == Rule::body {
            for it in child.into_inner() {
                // Parse item uniformly and determine if it is plain atomic
                let parsed = parse_item(it.clone());
                match parsed {
                    Item::ValueItem(Value::Atom(a)) => atoms.push(a),
                    Item::ValueItem(Value::Array(_))
                    | Item::ValueItem(Value::Block(_))
                    | Item::Pair { .. }
                    | Item::Comment(_) => {
                        items.push(parsed);
                        only_atoms = false;
                    }
                }
            }
        }
    }
    if only_atoms {
        Value::Array(atoms)
    } else {
        Value::Block(items)
    }
}

// Parse item (pair or plain value)
fn parse_item(p: Pair<Rule>) -> Item {
    match p.as_rule() {
        // Item is a wrapper node; inside is either a pair or a value
        Rule::item => {
            let mut inner = p.into_inner();
            if let Some(child) = inner.next() {
                return parse_item(child);
            }
            Item::ValueItem(Value::Atom(Atom::Ident(String::new())))
        }
        Rule::pair => {
            let mut it = p.into_inner();
            let key = parse_key(it.next().unwrap());
            let op = parse_operator(it.next().unwrap());
            let val_pair = it.next().unwrap();
            let value = match val_pair.as_rule() {
                // The third part of a pair is a wrapped value; unwrap into a concrete type
                Rule::value => {
                    let mut inner = val_pair.into_inner();
                    let v = inner.next().unwrap();
                    match v.as_rule() {
                        Rule::block => parse_block(v),
                        Rule::string
                        | Rule::date
                        | Rule::number
                        | Rule::boolean
                        | Rule::identifier => Value::Atom(parse_atom(v)),
                        _ => Value::Atom(Atom::Ident(v.as_str().to_string())),
                    }
                }
                Rule::block => parse_block(val_pair),
                Rule::string | Rule::date | Rule::number | Rule::boolean | Rule::identifier => {
                    Value::Atom(parse_atom(val_pair))
                }
                _ => Value::Atom(Atom::Ident(val_pair.as_str().to_string())),
            };
            Item::Pair { key, op, value }
        }
        Rule::value => {
            let mut inner = p.into_inner();
            let v = inner.next().unwrap();
            let val = match v.as_rule() {
                Rule::block => parse_block(v),
                Rule::string | Rule::date | Rule::number | Rule::boolean | Rule::identifier => {
                    Value::Atom(parse_atom(v))
                }
                _ => Value::Atom(Atom::Ident(v.as_str().to_string())),
            };
            Item::ValueItem(val)
        }
        Rule::comment => Item::Comment(p.as_str().to_string()),
        _ => Item::ValueItem(Value::Atom(Atom::Ident(p.as_str().to_string()))),
    }
}

// Parse file into a list of items
fn parse_file(pairs: Pairs<Rule>) -> Vec<Item> {
    let mut items = Vec::new();
    let file = pairs.into_iter().next().unwrap();
    for child in file.into_inner() {
        if child.as_rule() == Rule::body {
            for it in child.into_inner() {
                items.push(parse_item(it));
            }
        }
    }
    items
}

// Format date
fn fmt_date(d: &Date) -> String {
    match d.h {
        Some(h) => format!("\"{}.{}.{}.{}\"", d.y, d.m, d.d, h),
        None => format!("{}.{}.{}", d.y, d.m, d.d),
    }
}

// Serialize atomic value
fn serialize_atom(a: &Atom) -> String {
    match a {
        Atom::String(s) => format!("\"{}\"", s),
        Atom::Ident(s) => s.clone(),
        Atom::Number(n) => n.to_string(),
        Atom::Date(d) => fmt_date(d),
        Atom::Bool(b) => {
            if *b {
                "yes".to_string()
            } else {
                "no".to_string()
            }
        }
    }
}

// Serialize key
fn serialize_key(k: &KeyAtom) -> String {
    match k {
        KeyAtom::Ident(s) => s.clone(),
        KeyAtom::Number(n) => n.to_string(),
        KeyAtom::Date(d) => fmt_date(d),
    }
}

// Serialize value
fn serialize_value(v: &Value, indent: usize) -> String {
    match v {
        Value::Atom(a) => serialize_atom(a),
        Value::Array(arr) => {
            // Soft-wrap array at 120 characters (single element over 120 uses its own line)
            let rendered: Vec<String> = arr.iter().map(|a| serialize_atom(a)).collect();
            let mut out = String::new();
            out.push_str("{\n");
            let mut line = String::new();
            for (idx, elem) in rendered.iter().enumerate() {
                let sep = if line.is_empty() { "" } else { " " };
                let prospective_len = line.len() + sep.len() + elem.len();
                if !line.is_empty() && prospective_len > 120 {
                    out.push_str(&" ".repeat(indent + 2));
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
                // If it is the last element, output the current line
                if idx == rendered.len() - 1 {
                    out.push_str(&" ".repeat(indent + 2));
                    out.push_str(&line);
                    out.push('\n');
                }
            }
            out.push_str(&" ".repeat(indent));
            out.push_str("}\n");
            out
        }
        Value::Block(items) => {
            let mut out = String::new();
            out.push_str("{\n");
            for it in items {
                out.push_str(&serialize_item(it, indent + 4));
            }
            out.push_str(&" ".repeat(indent));
            out.push_str("}\n");
            out
        }
    }
}

// Serialize item
fn serialize_item(i: &Item, indent: usize) -> String {
    match i {
        Item::Pair { key, op, value } => {
            let mut line = String::new();
            line.push_str(&" ".repeat(indent));
            // TODO: 这里应该改为4空格长度制表符缩进，官方文件如此
            line.push_str(&serialize_key(key));
            line.push_str(" ");
            line.push_str(match op {
                Operator::Eq => "=",
                Operator::Le => "<=",
                Operator::Ge => ">=",
                Operator::Lt => "<",
                Operator::Gt => ">",
            });
            line.push_str(" ");
            match value {
                Value::Atom(_) => {
                    line.push_str(&serialize_value(value, indent));
                    line.push('\n');
                }
                Value::Array(_) => {
                    line.push_str(&serialize_value(value, indent));
                }
                Value::Block(_) => {
                    line.push_str(&serialize_value(value, indent));
                }
            }
            line
        }
        Item::ValueItem(v) => {
            let mut line = String::new();
            line.push_str(&" ".repeat(indent));
            match v {
                Value::Atom(_) => {
                    line.push_str(&serialize_value(v, indent));
                    line.push('\n');
                }
                Value::Array(_) => {
                    line.push_str(&serialize_value(v, indent));
                    line.push('\n');
                }
                Value::Block(_) => {
                    line.push_str(&serialize_value(v, indent));
                }
            }
            line
        }
        Item::Comment(s) => {
            let mut line = String::new();
            line.push_str(&" ".repeat(indent));
            line.push_str(s);
            line.push('\n');
            line
        }
    }
}
