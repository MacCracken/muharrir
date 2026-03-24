//! Expression evaluator for property fields.
//!
//! Wraps [`abaco::Evaluator`] so users can type math expressions
//! (e.g. `2*pi`, `sin(45)`, `sqrt(2)/2`) into any numeric editor field.

#[cfg(feature = "expr")]
use abaco::{EvalError, Evaluator, Value};

/// Evaluate a math expression string and return the result as `f64`.
#[cfg(feature = "expr")]
#[must_use = "expression result should be used"]
pub fn eval_f64(expr: &str) -> Result<f64, ExprError> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Err(ExprError::Empty);
    }
    let evaluator = Evaluator::new();
    let value = evaluator.eval(trimmed)?;
    let result = value.as_f64().ok_or(ExprError::NotNumeric(value));
    tracing::trace!(expr = trimmed, ok = result.is_ok(), "expression evaluated");
    result
}

/// Evaluate an expression, falling back to a default value on error.
#[cfg(feature = "expr")]
#[must_use]
#[inline]
pub fn eval_or(expr: &str, default: f64) -> f64 {
    eval_f64(expr).unwrap_or(default)
}

/// Try to evaluate an expression; if it fails, try parsing as a plain number.
#[cfg(feature = "expr")]
pub fn eval_or_parse(expr: &str) -> Result<f64, ExprError> {
    match eval_f64(expr) {
        Ok(v) => Ok(v),
        Err(_) => expr
            .trim()
            .parse::<f64>()
            .map_err(|_| ExprError::ParseFailed(expr.to_string())),
    }
}

/// Expression evaluation error.
#[cfg(feature = "expr")]
#[derive(Debug)]
#[non_exhaustive]
pub enum ExprError {
    /// Input string was empty or whitespace-only.
    Empty,
    /// Expression evaluated to a non-numeric value.
    NotNumeric(Value),
    /// Could not evaluate or parse the expression.
    ParseFailed(String),
    /// Evaluation error from abaco.
    Eval(EvalError),
}

#[cfg(feature = "expr")]
impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprError::Empty => write!(f, "empty expression"),
            ExprError::NotNumeric(v) => write!(f, "non-numeric result: {v}"),
            ExprError::ParseFailed(s) => write!(f, "cannot parse: {s}"),
            ExprError::Eval(e) => write!(f, "eval error: {e}"),
        }
    }
}

#[cfg(feature = "expr")]
impl std::error::Error for ExprError {}

#[cfg(feature = "expr")]
impl From<EvalError> for ExprError {
    fn from(e: EvalError) -> Self {
        ExprError::Eval(e)
    }
}

#[cfg(all(test, feature = "expr"))]
mod tests {
    use super::*;

    #[test]
    fn eval_arithmetic() {
        assert!((eval_f64("1 + 2 * 3").unwrap() - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_constants() {
        assert!((eval_f64("pi").unwrap() - std::f64::consts::PI).abs() < 1e-10);
        assert!((eval_f64("e").unwrap() - std::f64::consts::E).abs() < 1e-10);
        assert!((eval_f64("tau").unwrap() - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn eval_functions() {
        assert!((eval_f64("sqrt(9)").unwrap() - 3.0).abs() < f64::EPSILON);
        assert!(eval_f64("sin(0)").unwrap().abs() < 1e-10);
        assert!((eval_f64("abs(-42)").unwrap() - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_plain_number() {
        assert!((eval_f64("42.5").unwrap() - 42.5).abs() < 1e-10);
    }

    #[test]
    fn eval_empty() {
        assert!(matches!(eval_f64(""), Err(ExprError::Empty)));
        assert!(matches!(eval_f64("   "), Err(ExprError::Empty)));
    }

    #[test]
    fn eval_invalid() {
        assert!(eval_f64("+++").is_err());
        assert!(eval_f64("foobar(1)").is_err());
    }

    #[test]
    fn eval_or_fallback() {
        assert!((eval_or("1+1", 0.0) - 2.0).abs() < f64::EPSILON);
        assert!((eval_or("bad", 99.0) - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn eval_or_parse_works() {
        assert!((eval_or_parse("2*pi").unwrap() - std::f64::consts::TAU).abs() < 1e-10);
        assert!((eval_or_parse("42.5").unwrap() - 42.5).abs() < f64::EPSILON);
        assert!(eval_or_parse("not_a_number!@#").is_err());
    }
}
