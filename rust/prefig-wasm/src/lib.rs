//! WebAssembly bindings for PreFigure.
//!
//! What works today: the expression evaluator. The diagram builder
//! (`build_from_string`) is a placeholder until the drawing pipeline is ported;
//! it will keep the same signature and return `{ svg, annotations }` like the
//! Python `prefig.engine.build_from_string`.

use prefig_core::evaluator::ExpressionContext;
use prefig_core::value::Value;
use wasm_bindgen::prelude::*;

/// The crate version, for checking what is deployed.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Evaluates PreFigure math expressions and remembers definitions,
/// exactly like expressions written in diagram attributes.
#[wasm_bindgen]
pub struct Evaluator {
    ctx: ExpressionContext,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Evaluator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Evaluator {
        Evaluator {
            ctx: ExpressionContext::new(),
        }
    }

    /// Store a definition such as `"a = 5"` or `"f(x) = x^2 + 1"`.
    pub fn define(&mut self, expression: &str) -> Result<(), JsError> {
        self.ctx
            .define(expression)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    /// Evaluate an expression such as `"f(2) + a"` or `"(1,2) + (3,4)"`.
    /// Numbers, strings, and booleans come back as themselves; vectors come
    /// back as JavaScript arrays; dictionaries as plain objects.
    pub fn evaluate(&mut self, expression: &str) -> Result<JsValue, JsError> {
        let value = self
            .ctx
            .valid_eval(expression)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(value_to_js(&value))
    }
}

fn value_to_js(value: &Value) -> JsValue {
    match value {
        Value::Num(n) => JsValue::from_f64(*n),
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Str(s) => JsValue::from_str(s),
        Value::Array(items) => {
            let array = js_sys::Array::new();
            for item in items {
                array.push(&value_to_js(item));
            }
            array.into()
        }
        Value::Dict(map) => {
            let object = js_sys::Object::new();
            for (key, item) in map {
                let _ = js_sys::Reflect::set(
                    &object,
                    &JsValue::from_str(key),
                    &value_to_js(item),
                );
            }
            object.into()
        }
        Value::Function(_) => JsValue::from_str("<function>"),
    }
}

/// Build a diagram from PreFigure XML source. `format` is "svg" or "tactile".
///
/// Not implemented yet — the drawing pipeline is still being ported. Once it
/// lands this returns `{ svg: string, annotations: string | null }`, matching
/// what the Python version returns under Pyodide.
#[wasm_bindgen]
pub fn build_from_string(format: &str, source: &str) -> Result<JsValue, JsError> {
    let _ = (format, source);
    Err(JsError::new(
        "build_from_string is not implemented yet: the drawing pipeline is still being ported",
    ))
}
