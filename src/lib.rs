pub mod localisation;
pub mod script;

use crate::script::script::{parse_str, serialize_ast, Item};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn parse(content: &str) -> Result<JsValue, JsValue> {
    let ast = parse_str(content).map_err(|e| JsValue::from_str(&e))?;
    to_value(&ast).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn serialize(json: JsValue) -> Result<String, JsValue> {
    let ast: Vec<Item> = from_value(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let content = serialize_ast(&ast);
    Ok(content)
}
