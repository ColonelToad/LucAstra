//! LucAstra Calculator - Native arithmetic app
//!
//! Provides basic calculator operations: +, -, *, /, with support for
//! function calls (sin, cos, sqrt, etc.) and expression parsing.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CalcError {
    #[error("Division by zero")]
    DivideByZero,

    #[error("Invalid operation: {0}")]
    InvalidOp(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Math domain error: {0}")]
    DomainError(String),
}

pub type CalcResult<T> = Result<T, CalcError>;

/// Calculator state and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calculator {
    pub accumulator: f64,
    pub display: String,
    pub history: Vec<String>,
}

impl Default for Calculator {
    fn default() -> Self {
        Self {
            accumulator: 0.0,
            display: "0".to_string(),
            history: Vec::new(),
        }
    }
}

impl Calculator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse and evaluate an expression (e.g., "2 + 3 * 4")
    pub fn eval(&mut self, expr: &str) -> CalcResult<f64> {
        let expr = expr.trim();
        let result = self.parse_expression(expr)?;
        self.accumulator = result;
        self.display = format!("{}", result);
        self.history.push(format!("{} = {}", expr, result));
        Ok(result)
    }

    /// Basic expression parser supporting +, -, *, /, parentheses, and functions
    fn parse_expression(&self, expr: &str) -> CalcResult<f64> {
        // Tokenize
        let tokens = self.tokenize(expr)?;

        // Simple recursive descent parser
        let (result, _) = self.parse_additive(&tokens, 0)?;
        Ok(result)
    }

    /// Tokenize an expression into numbers, operators, functions, and parentheses
    fn tokenize(&self, expr: &str) -> CalcResult<Vec<String>> {
        let mut tokens = Vec::new();
        let mut current = String::new();

        for ch in expr.chars() {
            match ch {
                '+' | '-' | '*' | '/' | '(' | ')' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    tokens.push(ch.to_string());
                }
                ' ' => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        Ok(tokens)
    }

    /// Parse addition and subtraction (lowest precedence)
    fn parse_additive(&self, tokens: &[String], mut pos: usize) -> CalcResult<(f64, usize)> {
        let (mut left, new_pos) = self.parse_multiplicative(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match tokens[pos].as_str() {
                "+" => {
                    pos += 1;
                    let (right, new_pos) = self.parse_multiplicative(tokens, pos)?;
                    left += right;
                    pos = new_pos;
                }
                "-" => {
                    pos += 1;
                    let (right, new_pos) = self.parse_multiplicative(tokens, pos)?;
                    left -= right;
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    /// Parse multiplication and division (higher precedence)
    fn parse_multiplicative(&self, tokens: &[String], mut pos: usize) -> CalcResult<(f64, usize)> {
        let (mut left, new_pos) = self.parse_unary(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match tokens[pos].as_str() {
                "*" => {
                    pos += 1;
                    let (right, new_pos) = self.parse_unary(tokens, pos)?;
                    left *= right;
                    pos = new_pos;
                }
                "/" => {
                    pos += 1;
                    let (right, new_pos) = self.parse_unary(tokens, pos)?;
                    if right == 0.0 {
                        return Err(CalcError::DivideByZero);
                    }
                    left /= right;
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    /// Parse unary operations and function calls
    fn parse_unary(&self, tokens: &[String], pos: usize) -> CalcResult<(f64, usize)> {
        if pos >= tokens.len() {
            return Err(CalcError::ParseError(
                "Unexpected end of expression".to_string(),
            ));
        }

        match tokens[pos].as_str() {
            "-" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                Ok((-val, new_pos))
            }
            "+" => self.parse_primary(tokens, pos + 1),
            _ => self.parse_primary(tokens, pos),
        }
    }

    /// Parse primary terms: numbers, functions, and parenthesized expressions
    fn parse_primary(&self, tokens: &[String], pos: usize) -> CalcResult<(f64, usize)> {
        if pos >= tokens.len() {
            return Err(CalcError::ParseError(
                "Unexpected end of expression".to_string(),
            ));
        }

        match tokens[pos].as_str() {
            "(" => {
                let (val, new_pos) = self.parse_additive(tokens, pos + 1)?;
                if new_pos >= tokens.len() || tokens[new_pos] != ")" {
                    return Err(CalcError::ParseError("Expected ')'".to_string()));
                }
                Ok((val, new_pos + 1))
            }
            // Math functions
            "sqrt" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                if val < 0.0 {
                    return Err(CalcError::DomainError("sqrt of negative".to_string()));
                }
                Ok((val.sqrt(), new_pos))
            }
            "sin" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                Ok((val.sin(), new_pos))
            }
            "cos" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                Ok((val.cos(), new_pos))
            }
            "tan" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                Ok((val.tan(), new_pos))
            }
            "abs" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                Ok((val.abs(), new_pos))
            }
            "ln" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                if val <= 0.0 {
                    return Err(CalcError::DomainError("ln of non-positive".to_string()));
                }
                Ok((val.ln(), new_pos))
            }
            "log" => {
                let (val, new_pos) = self.parse_primary(tokens, pos + 1)?;
                if val <= 0.0 {
                    return Err(CalcError::DomainError("log of non-positive".to_string()));
                }
                Ok((val.log10(), new_pos))
            }
            // Number
            token => token
                .parse::<f64>()
                .map(|n| (n, pos + 1))
                .map_err(|_| CalcError::ParseError(format!("Invalid token: {}", token))),
        }
    }

    /// Clear accumulator and display
    pub fn clear(&mut self) {
        self.accumulator = 0.0;
        self.display = "0".to_string();
    }

    /// Get calculation history
    pub fn history(&self) -> &[String] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_addition() {
        let mut calc = Calculator::new();
        let result = calc.eval("2 + 3").unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_operator_precedence() {
        let mut calc = Calculator::new();
        let result = calc.eval("2 + 3 * 4").unwrap();
        assert_eq!(result, 14.0);
    }

    #[test]
    fn test_parentheses() {
        let mut calc = Calculator::new();
        let result = calc.eval("(2 + 3) * 4").unwrap();
        assert_eq!(result, 20.0);
    }

    #[test]
    fn test_division_by_zero() {
        let mut calc = Calculator::new();
        let result = calc.eval("5 / 0");
        assert!(matches!(result, Err(CalcError::DivideByZero)));
    }

    #[test]
    fn test_sqrt_function() {
        let mut calc = Calculator::new();
        let result = calc.eval("sqrt 16").unwrap();
        assert_eq!(result, 4.0);
    }

    #[test]
    fn test_sin_function() {
        let mut calc = Calculator::new();
        let result = calc.eval("sin 0").unwrap();
        assert!(result.abs() < 0.0001);
    }

    #[test]
    fn test_complex_expression() {
        let mut calc = Calculator::new();
        let result = calc.eval("(10 - 5) * 2 + 3").unwrap();
        assert_eq!(result, 13.0);
    }

    #[test]
    fn test_unary_minus() {
        let mut calc = Calculator::new();
        let result = calc.eval("-5 + 3").unwrap();
        assert_eq!(result, -2.0);
    }

    #[test]
    fn test_history() {
        let mut calc = Calculator::new();
        calc.eval("2 + 3").unwrap();
        calc.eval("5 * 2").unwrap();
        assert_eq!(calc.history.len(), 2);
        assert!(calc.history[0].contains("= 5"));
    }
}
