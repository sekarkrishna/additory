//! @calc mode - Calculate expressions
//!
//! Implements expression evaluation with arithmetic operators.

mod parser;

use crate::core::{DataFrame, AdditoryResult, AdditoryError, Expression, AsParam};
use polars::prelude::*;

/// Calculate new columns using expressions
///
/// # Parameters
/// - `df`: DataFrame to transform
/// - `expression`: Expression(s) to evaluate
/// - `name`: Output column name(s)
///
/// # Supported Operators
/// - Addition: `+`
/// - Subtraction: `-`
/// - Multiplication: `*`
/// - Division: `/`
/// - Modulo: `%`
/// - Power: `**`
/// - Grouping: `()`
///
/// Operator precedence and parenthesised grouping are fully supported.
/// Example: `weight / (height ** 2)` evaluates correctly.
///
/// # Returns
/// DataFrame with new calculated columns
pub fn calc(
    df: DataFrame,
    expression: Expression,
    name: Option<AsParam>,
) -> AdditoryResult<DataFrame> {
    match expression {
        Expression::Single(expr) => {
            // Single expression
            let col_name = match name {
                Some(AsParam::Single(name)) => name,
                Some(AsParam::Multiple(names)) if names.len() == 1 => names[0].clone(),
                None => "result".to_string(),
                _ => return Err(AdditoryError::validation(
                    "Single expression requires single output name",
                    "Provide name='column_name' or omit for default 'result'"
                )),
            };
            
            calc_single(&df, &expr, &col_name)
        }
        Expression::Multiple(exprs) => {
            // Multiple expressions
            let col_names = match name {
                Some(AsParam::Multiple(names)) => {
                    if names.len() != exprs.len() {
                        return Err(AdditoryError::validation(
                            &format!("Number of expressions ({}) must match number of output names ({})", 
                                    exprs.len(), names.len()),
                            "Provide same number of names as expressions"
                        ));
                    }
                    names
                }
                None => {
                    // Generate default names: result_0, result_1, ...
                    (0..exprs.len()).map(|i| format!("result_{}", i)).collect()
                }
                _ => return Err(AdditoryError::validation(
                    "Multiple expressions require multiple output names",
                    "Provide name=['name1', 'name2', ...]"
                )),
            };
            
            calc_multiple(&df, &exprs, &col_names)
        }
        Expression::Dict(_) => {
            Err(AdditoryError::validation(
                "Dict expressions not yet supported",
                "Use Expression::Single or Expression::Multiple"
            ))
        }
    }
}

/// Calculate single expression
fn calc_single(df: &DataFrame, expr: &str, col_name: &str) -> AdditoryResult<DataFrame> {
    let polars_expr = parser::parse_expression_new(expr, df)
        .map_err(|e| AdditoryError::operation(&e.to_string(), "Check expression syntax"))?;

    let result = df.inner().clone().lazy()
        .with_column(polars_expr.alias(col_name))
        .collect()
        .map_err(AdditoryError::Polars)?;

    Ok(DataFrame::from_polars(result))
}

/// Calculate multiple expressions, each step can reference previously added columns
fn calc_multiple(df: &DataFrame, exprs: &[String], col_names: &[String]) -> AdditoryResult<DataFrame> {
    let mut current = df.inner().clone();

    for (expr, col_name) in exprs.iter().zip(col_names.iter()) {
        let wrapper = DataFrame::from_polars(current.clone());
        let polars_expr = parser::parse_expression_new(expr, &wrapper)
            .map_err(|e| AdditoryError::operation(&e.to_string(), "Check expression syntax"))?;

        current = current.lazy()
            .with_column(polars_expr.alias(col_name))
            .collect()
            .map_err(AdditoryError::Polars)?;
    }

    Ok(DataFrame::from_polars(current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use polars::prelude::*;
    
    #[test]
    fn test_calc_single_addition() {
        let df_inner = df! {
            "a" => &[1, 2, 3],
            "b" => &[4, 5, 6],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let expr = Expression::Single("a + b".to_string());
        let name = Some(AsParam::Single("sum".to_string()));
        
        let result = calc(df, expr, name).unwrap();
        
        assert!(result.has_column("sum"));
        assert_eq!(result.height(), 3);
    }
    
    #[test]
    fn test_calc_single_multiplication() {
        let df_inner = df! {
            "price" => &[10.0, 20.0, 30.0],
            "quantity" => &[2, 3, 4],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let expr = Expression::Single("price * quantity".to_string());
        let name = Some(AsParam::Single("total".to_string()));
        
        let result = calc(df, expr, name).unwrap();
        
        assert!(result.has_column("total"));
    }
    
    #[test]
    fn test_calc_multiple_expressions() {
        let df_inner = df! {
            "a" => &[1, 2, 3],
            "b" => &[4, 5, 6],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let expr = Expression::Multiple(vec![
            "a + b".to_string(),
            "a * b".to_string(),
        ]);
        let name = Some(AsParam::Multiple(vec![
            "sum".to_string(),
            "product".to_string(),
        ]));
        
        let result = calc(df, expr, name).unwrap();
        
        assert!(result.has_column("sum"));
        assert!(result.has_column("product"));
    }
    
    #[test]
    fn test_calc_default_name() {
        let df_inner = df! {
            "a" => &[1, 2, 3],
            "b" => &[4, 5, 6],
        }.unwrap();

        let df = AdditoryDataFrame::from_polars(df_inner);
        let expr = Expression::Single("a + b".to_string());

        let result = calc(df, expr, None).unwrap();

        assert!(result.has_column("result"));
    }

    #[test]
    fn test_calc_parentheses_override_precedence() {
        // (a + b) * 2 should differ from a + b * 2
        let df_inner = df! {
            "a" => &[1.0, 2.0, 3.0],
            "b" => &[4.0, 5.0, 6.0],
        }.unwrap();

        let df = AdditoryDataFrame::from_polars(df_inner);
        let expr = Expression::Single("(a + b) * 2.0".to_string());
        let name = Some(AsParam::Single("result".to_string()));

        let result = calc(df, expr, name).unwrap();
        let col = result.inner().column("result").unwrap();
        let vals: Vec<f64> = col.f64().unwrap().into_no_null_iter().collect();
        // (1+4)*2=10, (2+5)*2=14, (3+6)*2=18
        assert_eq!(vals, vec![10.0, 14.0, 18.0]);
    }

    #[test]
    fn test_calc_bmi_formula() {
        // weight / (height ** 2) — canonical parentheses + power test
        let df_inner = df! {
            "weight" => &[70.0, 80.0, 90.0],
            "height" => &[1.75, 1.80, 1.70],
        }.unwrap();

        let df = AdditoryDataFrame::from_polars(df_inner);
        let expr = Expression::Single("weight / (height ** 2)".to_string());
        let name = Some(AsParam::Single("bmi".to_string()));

        let result = calc(df, expr, name).unwrap();
        assert!(result.has_column("bmi"));
        assert_eq!(result.height(), 3);
    }
}
