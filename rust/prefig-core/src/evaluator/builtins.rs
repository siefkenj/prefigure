//! Built-in functions available in author expressions.
//!
//! Mirrors what Python's user_namespace assembles from `math` plus
//! prefig/core/math_utilities.py. Functions that need the diagram (intersect,
//! proj_2d, …) arrive later with the EvalEnv handle (outline §4.1).

use super::interp::call_value;
use super::{EvalError, ExpressionContext};
use crate::core::calculus;
use crate::value::Value;

fn err(m: impl Into<String>) -> EvalError {
    EvalError::new(m)
}

fn arity(name: &str, args: &[Value], n: usize) -> Result<(), EvalError> {
    if args.len() != n {
        return Err(err(format!("{name}() takes {n} arguments, got {}", args.len())));
    }
    Ok(())
}

fn num(name: &str, args: &[Value], i: usize) -> Result<f64, EvalError> {
    args.get(i)
        .ok_or_else(|| err(format!("{name}(): missing argument {i}")))?
        .as_num()
}

fn one_num(name: &str, args: &[Value], f: impl Fn(f64) -> f64) -> Result<Value, EvalError> {
    arity(name, args, 1)?;
    Ok(Value::Num(f(num(name, args, 0)?)))
}

fn two_num(name: &str, args: &[Value], f: impl Fn(f64, f64) -> f64) -> Result<Value, EvalError> {
    arity(name, args, 2)?;
    Ok(Value::Num(f(num(name, args, 0)?, num(name, args, 1)?)))
}

const NAMES: &[&str] = &[
    // from math
    "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh",
    "asinh", "acosh", "atanh", "exp", "log", "log2", "log10", "sqrt", "floor",
    "ceil", "degrees", "radians", "factorial", "comb", "fabs", "hypot", "copysign",
    "trunc", "isclose", "gcd", "pow",
    // python builtins whitelisted in user_namespace
    "max", "min", "round", "abs",
    // math_utilities
    "ln", "sec", "csc", "cot", "dot", "distance", "length", "normalize",
    "midpoint", "angle", "roll", "choose", "append", "chi_oo", "chi_oc", "chi_co",
    "chi_cc", "rotate", "deriv", "zip_lists", "evaluate_bezier", "eulers_method",
];

pub fn is_builtin(name: &str) -> bool {
    NAMES.contains(&name)
}

pub fn call(name: &str, args: &[Value], ctx: &mut ExpressionContext) -> Result<Value, EvalError> {
    match name {
        "sin" => one_num(name, args, f64::sin),
        "cos" => one_num(name, args, f64::cos),
        "tan" => one_num(name, args, f64::tan),
        "asin" => one_num(name, args, f64::asin),
        "acos" => one_num(name, args, f64::acos),
        "atan" => one_num(name, args, f64::atan),
        "atan2" => two_num(name, args, f64::atan2),
        "sinh" => one_num(name, args, f64::sinh),
        "cosh" => one_num(name, args, f64::cosh),
        "tanh" => one_num(name, args, f64::tanh),
        "asinh" => one_num(name, args, f64::asinh),
        "acosh" => one_num(name, args, f64::acosh),
        "atanh" => one_num(name, args, f64::atanh),
        "exp" => one_num(name, args, f64::exp),
        "ln" => one_num(name, args, f64::ln),
        "log" => match args.len() {
            1 => one_num(name, args, f64::ln),
            2 => two_num(name, args, |x, base| x.log(base)),
            n => Err(err(format!("log() takes 1 or 2 arguments, got {n}"))),
        },
        "log2" => one_num(name, args, f64::log2),
        "log10" => one_num(name, args, f64::log10),
        "sqrt" => one_num(name, args, f64::sqrt),
        "floor" => one_num(name, args, f64::floor),
        "ceil" => one_num(name, args, f64::ceil),
        "degrees" => one_num(name, args, f64::to_degrees),
        "radians" => one_num(name, args, f64::to_radians),
        "fabs" => one_num(name, args, f64::abs),
        "trunc" => one_num(name, args, f64::trunc),
        "hypot" => two_num(name, args, f64::hypot),
        "copysign" => two_num(name, args, f64::copysign),
        "pow" => two_num(name, args, f64::powf),
        "factorial" => one_num(name, args, |n| (2..=(n as u64)).product::<u64>() as f64),
        "comb" | "choose" => {
            arity(name, args, 2)?;
            let (n, k) = (num(name, args, 0)? as u64, num(name, args, 1)? as u64);
            Ok(Value::Num(binomial(n, k)))
        }
        "gcd" => two_num(name, args, |a, b| {
            let (mut a, mut b) = ((a as i64).unsigned_abs(), (b as i64).unsigned_abs());
            while b != 0 {
                (a, b) = (b, a % b);
            }
            a as f64
        }),
        "isclose" => {
            arity(name, args, 2)?;
            let (a, b) = (num(name, args, 0)?, num(name, args, 1)?);
            // math.isclose defaults: rel_tol=1e-9, abs_tol=0
            Ok(Value::Bool((a - b).abs() <= 1e-9 * a.abs().max(b.abs())))
        }
        "sec" => one_num(name, args, |x| 1.0 / x.cos()),
        "csc" => one_num(name, args, |x| 1.0 / x.sin()),
        "cot" => one_num(name, args, |x| 1.0 / x.tan()),

        "abs" => {
            arity(name, args, 1)?;
            abs_value(&args[0])
        }
        "max" | "min" => {
            if args.is_empty() {
                return Err(err(format!("{name}() needs at least one argument")));
            }
            // max((1,2,3)) over a single vector, or max(1, 2, 3) over scalars
            let nums: Vec<f64> = if args.len() == 1 {
                args[0].as_vec_f64()?
            } else {
                args.iter().map(|v| v.as_num()).collect::<Result<_, _>>()?
            };
            let init = nums[0];
            let folded = nums.into_iter().fold(init, |acc, x| {
                if name == "max" {
                    acc.max(x)
                } else {
                    acc.min(x)
                }
            });
            Ok(Value::Num(folded))
        }
        "round" => match args.len() {
            // Python round() is banker's rounding
            1 => one_num(name, args, f64::round_ties_even),
            2 => two_num(name, args, |x, nd| {
                if nd >= 0.0 {
                    // Python rounds the true decimal value (half-to-even); Rust's
                    // float formatter does exactly that, while multiply-by-10^n
                    // would introduce binary error (round(2.675, 2) must be 2.67)
                    format!("{x:.*}", nd as usize).parse().unwrap_or(x)
                } else {
                    let scale = 10f64.powi(nd as i32);
                    (x * scale).round_ties_even() / scale
                }
            }),
            n => Err(err(format!("round() takes 1 or 2 arguments, got {n}"))),
        },

        "dot" => {
            arity(name, args, 2)?;
            let (u, v) = (args[0].as_vec_f64()?, args[1].as_vec_f64()?);
            if u.len() != v.len() {
                return Err(err("dot(): vectors have different lengths"));
            }
            Ok(Value::Num(u.iter().zip(&v).map(|(a, b)| a * b).sum()))
        }
        "length" => {
            arity(name, args, 1)?;
            Ok(Value::Num(norm(&args[0].as_vec_f64()?)))
        }
        "distance" => {
            arity(name, args, 2)?;
            let (p, q) = (args[0].as_vec_f64()?, args[1].as_vec_f64()?);
            if p.len() != q.len() {
                return Err(err("distance(): points have different dimensions"));
            }
            let diff: Vec<f64> = p.iter().zip(&q).map(|(a, b)| a - b).collect();
            Ok(Value::Num(norm(&diff)))
        }
        "normalize" => {
            arity(name, args, 1)?;
            let u = args[0].as_vec_f64()?;
            let n = norm(&u);
            if n == 0.0 {
                return Err(err("normalize(): zero vector"));
            }
            Ok(nums_to_value(u.iter().map(|x| x / n)))
        }
        "midpoint" => {
            arity(name, args, 2)?;
            let (u, v) = (args[0].as_vec_f64()?, args[1].as_vec_f64()?);
            Ok(nums_to_value(u.iter().zip(&v).map(|(a, b)| 0.5 * (a + b))))
        }
        "angle" => {
            let p = args
                .first()
                .ok_or_else(|| err("angle() needs a point"))?
                .as_vec_f64()?;
            let radians = p[1].atan2(p[0]);
            let degrees_wanted = match args.get(1) {
                None => true,
                Some(Value::Str(s)) => s == "deg",
                Some(other) => return Err(err(format!("angle(): bad units {other:?}"))),
            };
            Ok(Value::Num(if degrees_wanted {
                radians.to_degrees()
            } else {
                radians
            }))
        }
        "rotate" => {
            arity(name, args, 2)?;
            let v = args[0].as_vec_f64()?;
            let theta = num(name, args, 1)?;
            let (c, s) = (theta.cos(), theta.sin());
            Ok(nums_to_value([c * v[0] - s * v[1], s * v[0] + c * v[1]].into_iter()))
        }
        "roll" => {
            arity(name, args, 1)?;
            match &args[0] {
                Value::Array(items) if !items.is_empty() => {
                    let mut rolled = items.clone();
                    rolled.rotate_right(1);
                    Ok(Value::Array(rolled))
                }
                other => Err(err(format!("roll(): expected an array, found {other:?}"))),
            }
        }
        "append" => {
            arity(name, args, 2)?;
            match &args[0] {
                Value::Array(items) => {
                    let mut out = items.clone();
                    out.push(args[1].clone());
                    Ok(Value::Array(out))
                }
                other => Err(err(format!("append(): expected an array, found {other:?}"))),
            }
        }
        "zip_lists" => {
            arity(name, args, 2)?;
            match (&args[0], &args[1]) {
                (Value::Array(a), Value::Array(b)) => Ok(Value::Array(
                    a.iter()
                        .zip(b)
                        .map(|(x, y)| Value::Array(vec![x.clone(), y.clone()]))
                        .collect(),
                )),
                _ => Err(err("zip_lists(): expected two arrays")),
            }
        }
        "chi_oo" | "chi_oc" | "chi_co" | "chi_cc" => {
            arity(name, args, 3)?;
            let (a, b, t) = (num(name, args, 0)?, num(name, args, 1)?, num(name, args, 2)?);
            let lower = if name.as_bytes()[4] == b'o' { t > a } else { t >= a };
            let upper = if name.as_bytes()[5] == b'o' { t < b } else { t <= b };
            Ok(Value::Num(if lower && upper { 1.0 } else { 0.0 }))
        }
        "evaluate_bezier" => {
            arity(name, args, 2)?;
            evaluate_bezier(&args[0], num(name, args, 1)?)
        }
        "deriv" => {
            arity(name, args, 2)?;
            let f = args[0].clone();
            let a = num(name, args, 1)?;
            let d = calculus::derivative(
                |x| call_value(&f, &[Value::Num(x)], ctx)?.as_num(),
                a,
                true,
            )?;
            Ok(Value::Num(d))
        }
        "eulers_method" => {
            arity(name, args, 5)?;
            eulers_method(
                &args[0],
                num(name, args, 1)?,
                args[2].clone(),
                num(name, args, 3)?,
                num(name, args, 4)? as usize,
                ctx,
            )
        }
        _ => Err(err(format!("Unknown function in evaluation: {name}"))),
    }
}

fn norm(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn nums_to_value(iter: impl Iterator<Item = f64>) -> Value {
    Value::Array(iter.map(Value::Num).collect())
}

fn binomial(n: u64, k: u64) -> f64 {
    if k > n {
        return 0.0;
    }
    let k = k.min(n - k);
    let mut result = 1f64;
    for i in 0..k {
        result = result * (n - i) as f64 / (i + 1) as f64;
    }
    result.round()
}

fn abs_value(v: &Value) -> Result<Value, EvalError> {
    // Python's abs() on an ndarray is elementwise
    match v {
        Value::Array(items) => {
            let items: Result<Vec<_>, _> = items.iter().map(abs_value).collect();
            Ok(Value::Array(items?))
        }
        _ => Ok(Value::Num(v.as_num()?.abs())),
    }
}

/// Port of math_utilities.evaluate_bezier (quadratic and cubic).
fn evaluate_bezier(controls: &Value, t: f64) -> Result<Value, EvalError> {
    let controls = match controls {
        Value::Array(items) => items,
        other => return Err(err(format!("evaluate_bezier(): bad controls {other:?}"))),
    };
    let n = controls.len();
    let coefficients: &[f64] = match n {
        3 => &[1.0, 2.0, 1.0],
        4 => &[1.0, 3.0, 3.0, 1.0],
        _ => return Err(err("evaluate_bezier(): need 3 or 4 control points")),
    };
    let dim = controls[0].as_vec_f64()?.len();
    let mut sum = vec![0.0; dim];
    for (j, control) in controls.iter().enumerate() {
        let point = control.as_vec_f64()?;
        let weight =
            coefficients[j] * (1.0 - t).powi((n - j - 1) as i32) * t.powi(j as i32);
        for (acc, c) in sum.iter_mut().zip(&point) {
            *acc += weight * c;
        }
    }
    Ok(nums_to_value(sum.into_iter()))
}

/// Port of math_utilities.eulers_method: rows are [t, *y].
fn eulers_method(
    f: &Value,
    t0: f64,
    y0: Value,
    t1: f64,
    n: usize,
    ctx: &mut ExpressionContext,
) -> Result<Value, EvalError> {
    use crate::evaluator::ast::BinOp;
    let h = (t1 - t0) / n as f64;
    let row = |t: f64, y: &Value| -> Value {
        let mut cells = vec![Value::Num(t)];
        match y {
            Value::Array(items) => cells.extend(items.iter().cloned()),
            other => cells.push(other.clone()),
        }
        Value::Array(cells)
    };

    let mut t = t0;
    let mut y = y0;
    let mut points = vec![row(t, &y)];
    for _ in 0..n {
        let dy = call_value(f, &[Value::Num(t), y.clone()], ctx)?;
        let step = crate::value::binop(BinOp::Mult, &dy, &Value::Num(h))?;
        y = crate::value::binop(BinOp::Add, &y, &step)?;
        t += h;
        points.push(row(t, &y));
    }
    Ok(Value::Array(points))
}
