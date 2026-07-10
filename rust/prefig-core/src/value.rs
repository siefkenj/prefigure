//! Dynamically-typed value with numpy-like semantics (RUST_PORT_OUTLINE.md §6.2).
//!
//! Author expressions in Python evaluate over numpy arrays; this type reproduces
//! the behaviors PreFigure actually uses: elementwise arithmetic with
//! trailing-dimension broadcasting, negative and "fancy" (index-array) indexing,
//! and ragged nested arrays (Python's inhomogeneous-array fallback).

use crate::evaluator::ast::{BinOp, Expr, UnaryOp};
use crate::evaluator::EvalError;
use indexmap::IndexMap;
use std::rc::Rc;

#[derive(Clone)]
pub enum Value {
    Num(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Dict(IndexMap<String, Value>),
    Function(Rc<Function>),
}

pub enum Function {
    /// Author-defined `f(x) = …`: the body re-resolves names at call time,
    /// exactly like the Python lambda over globals().
    User { params: Vec<String>, body: Expr },
    /// A built-in registered by name (see evaluator/builtins.rs).
    Native(&'static str),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{n:?}"),
            Value::Bool(b) => write!(f, "{b:?}"),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Array(items) => f.debug_list().entries(items).finish(),
            Value::Dict(map) => f.debug_map().entries(map.iter()).finish(),
            Value::Function(func) => match func.as_ref() {
                Function::User { params, .. } => write!(f, "<function({})>", params.join(", ")),
                Function::Native(name) => write!(f, "<builtin {name}>"),
            },
        }
    }
}

fn err(message: impl Into<String>) -> EvalError {
    EvalError::new(message)
}

impl Value {
    pub fn as_num(&self) -> Result<f64, EvalError> {
        match self {
            Value::Num(n) => Ok(*n),
            // Python bools are ints; True + 1 == 2
            Value::Bool(b) => Ok(*b as u8 as f64),
            other => Err(err(format!("expected a number, found {other:?}"))),
        }
    }

    pub fn as_index(&self, len: usize) -> Result<usize, EvalError> {
        let n = self.as_num()?;
        if n.fract() != 0.0 {
            return Err(err(format!("index {n} is not an integer")));
        }
        let i = n as i64;
        let wrapped = if i < 0 { i + len as i64 } else { i };
        if wrapped < 0 || wrapped >= len as i64 {
            return Err(err(format!("index {i} out of range for length {len}")));
        }
        Ok(wrapped as usize)
    }

    pub fn as_vec_f64(&self) -> Result<Vec<f64>, EvalError> {
        match self {
            Value::Array(items) => items.iter().map(|v| v.as_num()).collect(),
            other => Err(err(format!("expected a vector, found {other:?}"))),
        }
    }

    /// Nesting depth: 0 for scalars, 1 for vectors, 2 for lists of points, …
    /// (numpy's ndim; ragged arrays use the maximum over items).
    pub fn rank(&self) -> usize {
        match self {
            Value::Array(items) => 1 + items.iter().map(Value::rank).max().unwrap_or(0),
            _ => 0,
        }
    }

    /// Python's str() of a dict key (int keys print without a decimal point).
    pub fn as_dict_key(&self) -> Result<String, EvalError> {
        match self {
            Value::Str(s) => Ok(s.clone()),
            Value::Num(n) if n.fract() == 0.0 && n.is_finite() => Ok(format!("{}", *n as i64)),
            Value::Num(n) => Ok(format!("{n}")),
            Value::Bool(b) => Ok(if *b { "True" } else { "False" }.to_string()),
            other => Err(err(format!("invalid dict key: {other:?}"))),
        }
    }
}

fn scalar_binop(op: BinOp, a: f64, b: f64) -> Result<f64, EvalError> {
    match op {
        BinOp::Add => Ok(a + b),
        BinOp::Sub => Ok(a - b),
        BinOp::Mult => Ok(a * b),
        BinOp::Div => {
            if b == 0.0 {
                Err(err("division by zero"))
            } else {
                Ok(a / b)
            }
        }
        BinOp::FloorDiv => {
            if b == 0.0 {
                Err(err("division by zero"))
            } else {
                Ok((a / b).floor())
            }
        }
        // Python semantics: result has the sign of the divisor
        BinOp::Mod => {
            if b == 0.0 {
                Err(err("modulo by zero"))
            } else {
                Ok(a - b * (a / b).floor())
            }
        }
        BinOp::Pow => Ok(a.powf(b)),
    }
}

/// Elementwise arithmetic with numpy-style trailing-dimension broadcasting:
/// the higher-rank operand maps the lower-rank one over its items, so
/// `[[1,2],[3,4]] + [10,20]` adds `[10,20]` to each row.
pub fn binop(op: BinOp, a: &Value, b: &Value) -> Result<Value, EvalError> {
    match (a, b) {
        (Value::Array(xs), Value::Array(ys)) => {
            let (ra, rb) = (a.rank(), b.rank());
            if ra > rb {
                let items: Result<Vec<_>, _> = xs.iter().map(|x| binop(op, x, b)).collect();
                Ok(Value::Array(items?))
            } else if rb > ra {
                let items: Result<Vec<_>, _> = ys.iter().map(|y| binop(op, a, y)).collect();
                Ok(Value::Array(items?))
            } else if xs.len() == ys.len() {
                let items: Result<Vec<_>, _> =
                    xs.iter().zip(ys).map(|(x, y)| binop(op, x, y)).collect();
                Ok(Value::Array(items?))
            } else {
                Err(err(format!(
                    "operands have mismatched lengths {} and {}",
                    xs.len(),
                    ys.len()
                )))
            }
        }
        (Value::Array(xs), _) => {
            let items: Result<Vec<_>, _> = xs.iter().map(|x| binop(op, x, b)).collect();
            Ok(Value::Array(items?))
        }
        (_, Value::Array(ys)) => {
            let items: Result<Vec<_>, _> = ys.iter().map(|y| binop(op, a, y)).collect();
            Ok(Value::Array(items?))
        }
        _ => Ok(Value::Num(scalar_binop(op, a.as_num()?, b.as_num()?)?)),
    }
}

pub fn unop(op: UnaryOp, v: &Value) -> Result<Value, EvalError> {
    match v {
        Value::Array(items) => {
            let items: Result<Vec<_>, _> = items.iter().map(|x| unop(op, x)).collect();
            Ok(Value::Array(items?))
        }
        _ => {
            let n = v.as_num()?;
            Ok(Value::Num(match op {
                UnaryOp::Neg => -n,
                UnaryOp::Pos => n,
            }))
        }
    }
}

/// Indexing. A scalar index selects one element (negative wraps, as in Python).
/// An array index does numpy "fancy" indexing — it selects per element — which
/// is what Python-PreFigure's `m[i, j]` actually does: the AST rewrite wraps the
/// index tuple in np.array, so `m[1, 0]` yields rows 1 and 0, not element [1][0].
pub fn subscript(target: &Value, index: &Value) -> Result<Value, EvalError> {
    match target {
        Value::Array(items) => match index {
            Value::Array(idxs) => {
                let selected: Result<Vec<_>, _> =
                    idxs.iter().map(|i| subscript(target, i)).collect();
                Ok(Value::Array(selected?))
            }
            _ => Ok(items[index.as_index(items.len())?].clone()),
        },
        Value::Dict(map) => {
            let key = index.as_dict_key()?;
            map.get(&key)
                .cloned()
                .ok_or_else(|| err(format!("key {key:?} not found")))
        }
        other => Err(err(format!("{other:?} is not subscriptable"))),
    }
}
