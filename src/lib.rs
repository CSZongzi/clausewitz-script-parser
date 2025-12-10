pub mod parser;

use crate::parser::parse_str;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn parse(content: &str) -> Result<JsValue, JsValue> {
    let ast = parse_str(content).map_err(|e| JsValue::from_str(&e))?;
    to_value(&ast).map_err(|e| JsValue::from_str(&e.to_string()))
}
