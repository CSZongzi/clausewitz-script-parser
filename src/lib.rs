pub mod localisation;
pub mod script;
pub mod string_utils;

use crate::localisation::localisation::{
    parse_str as parse_loc_str, serialize_ast as serialize_loc_ast,
};
use crate::script::script::{parse_str as parse_scr_str, serialize_ast as serialize_scr_ast, Item};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub struct Script;

#[wasm_bindgen]
impl Script {
    /// 从字符串解析 AST
    ///
    /// # Arguments
    ///
    /// * `content`: 文件内容
    ///
    /// returns: Result<JsValue, JsValue>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    #[wasm_bindgen]
    pub fn parse(content: &str) -> Result<JsValue, JsValue> {
        let ast = parse_scr_str(content).map_err(|e| JsValue::from_str(&e))?;
        to_value(&ast).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// 序列化 AST 为字符串
    ///
    /// # Arguments
    ///
    /// * `json`: 条目列表
    ///
    /// returns: Result<String, JsValue>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    #[wasm_bindgen]
    pub fn serialize(json: JsValue) -> Result<String, JsValue> {
        let ast: Vec<Item> = from_value(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let content = serialize_scr_ast(&ast);
        Ok(content)
    }
}

#[wasm_bindgen]
pub struct Localisation;

#[wasm_bindgen]
impl Localisation {
    /// 根据字符串解析 AST
    ///
    /// # Arguments
    ///
    /// * `content`: 文件内容
    ///
    /// returns: Result<JsValue, JsValue>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    #[wasm_bindgen]
    pub fn parse(content: &str) -> Result<JsValue, JsValue> {
        let ast = parse_loc_str(content).map_err(|e| JsValue::from_str(&e))?;
        to_value(&ast).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// 将 AST 序列化为本地化内容
    ///
    /// # Arguments
    ///
    /// * `json`: AST
    ///
    /// returns: Result<String, JsValue>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    #[wasm_bindgen]
    pub fn serialize(json: JsValue) -> Result<String, JsValue> {
        let ast = from_value(json).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let content = serialize_loc_ast(&ast);
        Ok(content)
    }
}
