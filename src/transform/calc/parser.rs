//! Expression parser for @calc mode
//!
//! Implements a robust expression parser with support for:
//! - Parentheses for grouping: (a + b) * c
//! - Power operator: height ** 2
//! - Proper operator precedence (PEMDAS)
//! - Clear error messages with position information
//!
//! ## Architecture
//!
//! The parser uses a three-stage pipeline:
//! 1. **Tokenizer**: Converts expression string into tokens
//! 2. **Pratt Parser**: Builds Abstract Syntax Tree (AST) using binding powers
//! 3. **AST Evaluator**: Converts AST to Polars expressions
//!
//! ## Example
//!
//! ```rust,ignore
//! let df = create_test_df();
//! let expr = parse_expression("weight / (height ** 2)", &df)?;
//! ```

use crate::core::AdditoryError;
use std::fmt;

/// Parse errors with position information for clear diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Invalid character encountered during tokenization
    InvalidCharacter { ch: char, position: usize },
    
    /// Malformed numeric literal
    InvalidNumber { text: String, position: usize },
    
    /// Unexpected token during parsing
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    
    /// Unbalanced parentheses
    UnbalancedParentheses { position: usize, message: String },
    
    /// Empty expression or empty parentheses
    EmptyExpression { message: String },
    
    /// Missing operand for operator
    #[allow(dead_code)]
    MissingOperand { operator: String, position: usize },
    
    /// Column not found in DataFrame
    ColumnNotFound {
        column: String,
        available: Vec<String>,
    },
    
    /// Unexpected end of input
    UnexpectedEof { expected: String },
}

impl ParseError {
    /// Convert ParseError to AdditoryError with formatted message
    ///
    /// Generates user-friendly error messages with:
    /// - Original expression
    /// - Position indicator (^)
    /// - Error description
    /// - Suggestion for fixing
    #[allow(dead_code)]
    pub fn to_additory_error(&self, expression: &str) -> AdditoryError {
        match self {
            ParseError::InvalidCharacter { ch, position } => {
                let indicator = format!("{:>width$}", "^", width = position + 1);
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\n{}\nError: Invalid character '{}' at position {}",
                        expression, indicator, ch, position
                    ),
                    "Remove invalid characters. Valid characters: a-z, A-Z, 0-9, +, -, *, /, %, (, ), ., _".to_string(),
                )
            }
            
            ParseError::InvalidNumber { text, position } => {
                let indicator = format!("{:>width$}", "^", width = position + 1);
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\n{}\nError: Invalid number '{}' at position {}",
                        expression, indicator, text, position
                    ),
                    "Check number format. Valid formats: 42, 3.14, 1.5e-10".to_string(),
                )
            }
            
            ParseError::UnexpectedToken { expected, found, position } => {
                let indicator = format!("{:>width$}", "^", width = position + 1);
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\n{}\nError: Expected {} but found {} at position {}",
                        expression, indicator, expected, found, position
                    ),
                    format!("Replace {} with {}", found, expected),
                )
            }
            
            ParseError::UnbalancedParentheses { position, message } => {
                let indicator = format!("{:>width$}", "^", width = position + 1);
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\n{}\nError: {}",
                        expression, indicator, message
                    ),
                    "Ensure all opening parentheses '(' have matching closing parentheses ')'".to_string(),
                )
            }
            
            ParseError::EmptyExpression { message } => {
                AdditoryError::Operation(
                    format!("Failed to parse expression: {}\nError: {}", expression, message),
                    "Provide a non-empty expression".to_string(),
                )
            }
            
            ParseError::MissingOperand { operator, position } => {
                let indicator = format!("{:>width$}", "^", width = position + 1);
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\n{}\nError: Operator '{}' is missing an operand at position {}",
                        expression, indicator, operator, position
                    ),
                    format!("Add an operand before or after '{}'", operator),
                )
            }
            
            ParseError::ColumnNotFound { column, available } => {
                let suggestion = if available.is_empty() {
                    "DataFrame has no columns".to_string()
                } else {
                    format!("Available columns: {}", available.join(", "))
                };
                
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\nError: Column '{}' not found in DataFrame",
                        expression, column
                    ),
                    suggestion,
                )
            }
            
            ParseError::UnexpectedEof { expected } => {
                AdditoryError::Operation(
                    format!(
                        "Failed to parse expression: {}\nError: Unexpected end of expression, expected {}",
                        expression, expected
                    ),
                    format!("Add {} at the end of the expression", expected),
                )
            }
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidCharacter { ch, position } => {
                write!(f, "Invalid character '{}' at position {}", ch, position)
            }
            ParseError::InvalidNumber { text, position } => {
                write!(f, "Invalid number '{}' at position {}", text, position)
            }
            ParseError::UnexpectedToken { expected, found, position } => {
                write!(f, "Expected {} but found {} at position {}", expected, found, position)
            }
            ParseError::UnbalancedParentheses { position, message } => {
                write!(f, "{} at position {}", message, position)
            }
            ParseError::EmptyExpression { message } => {
                write!(f, "{}", message)
            }
            ParseError::MissingOperand { operator, position } => {
                write!(f, "Operator '{}' is missing an operand at position {}", operator, position)
            }
            ParseError::ColumnNotFound { column, .. } => {
                write!(f, "Column '{}' not found", column)
            }
            ParseError::UnexpectedEof { expected } => {
                write!(f, "Unexpected end of expression, expected {}", expected)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Token types for expression parsing
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    /// Numeric literal (integer or float)
    Number(f64),
    
    /// Identifier (column name)
    Ident(&'a str),
    
    /// Addition operator (+)
    Plus,
    
    /// Subtraction operator (-)
    Minus,
    
    /// Multiplication operator (*)
    Star,
    
    /// Division operator (/)
    Slash,
    
    /// Modulo operator (%)
    Percent,
    
    /// Power operator (**)
    Power,
    
    /// Left parenthesis (()
    LParen,
    
    /// Right parenthesis ())
    RParen,
    
    /// End of input
    Eof,
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "number {}", n),
            Token::Ident(s) => write!(f, "identifier '{}'", s),
            Token::Plus => write!(f, "'+'"),
            Token::Minus => write!(f, "'-'"),
            Token::Star => write!(f, "'*'"),
            Token::Slash => write!(f, "'/'"),
            Token::Percent => write!(f, "'%'"),
            Token::Power => write!(f, "'**'"),
            Token::LParen => write!(f, "'('"),
            Token::RParen => write!(f, "')'"),
            Token::Eof => write!(f, "end of input"),
        }
    }
}

/// Tokenizer for expression strings
///
/// Converts expression strings into a stream of tokens for parsing.
/// Tracks position for error reporting.
pub struct Tokenizer<'a> {
    /// Input expression string
    input: &'a str,
    
    /// Current position in input (byte offset)
    position: usize,
    
    /// Current character being examined
    current_char: Option<char>,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer for the given expression
    pub fn new(input: &'a str) -> Self {
        let mut tokenizer = Self {
            input,
            position: 0,
            current_char: None,
        };
        tokenizer.current_char = tokenizer.input.chars().next();
        tokenizer
    }
    
    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Token<'a>, ParseError> {
        self.skip_whitespace();
        
        let start_pos = self.position;
        
        match self.current_char {
            None => Ok(Token::Eof),
            
            Some('(') => {
                self.advance();
                Ok(Token::LParen)
            }
            
            Some(')') => {
                self.advance();
                Ok(Token::RParen)
            }
            
            Some('+') => {
                self.advance();
                Ok(Token::Plus)
            }
            
            Some('-') => {
                self.advance();
                Ok(Token::Minus)
            }
            
            Some('*') => {
                self.advance();
                // Check for ** (power operator)
                if self.current_char == Some('*') {
                    self.advance();
                    Ok(Token::Power)
                } else {
                    Ok(Token::Star)
                }
            }
            
            Some('/') => {
                self.advance();
                Ok(Token::Slash)
            }
            
            Some('%') => {
                self.advance();
                Ok(Token::Percent)
            }
            
            Some(ch) if ch.is_ascii_digit() || ch == '.' => {
                self.read_number()
            }
            
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                Ok(Token::Ident(self.read_identifier()))
            }
            
            Some(ch) => {
                Err(ParseError::InvalidCharacter {
                    ch,
                    position: start_pos,
                })
            }
        }
    }
    
    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// Advance to the next character
    fn advance(&mut self) {
        if self.current_char.is_some() {
            self.position += 1;
            self.current_char = self.input[self.position..].chars().next();
        }
    }
    
    /// Read a numeric literal (integer or float)
    fn read_number(&mut self) -> Result<Token<'a>, ParseError> {
        let start_pos = self.position;
        let mut has_dot = false;
        let mut has_e = false;
        
        // Read digits before decimal point
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot && !has_e {
                has_dot = true;
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !has_e {
                has_e = true;
                self.advance();
                // Handle optional sign after 'e'
                if let Some('+') | Some('-') = self.current_char {
                    self.advance();
                }
            } else {
                break;
            }
        }
        
        let number_str = &self.input[start_pos..self.position];
        
        number_str.parse::<f64>()
            .map(Token::Number)
            .map_err(|_| ParseError::InvalidNumber {
                text: number_str.to_string(),
                position: start_pos,
            })
    }
    
    /// Read an identifier (column name)
    fn read_identifier(&mut self) -> &'a str {
        let start_pos = self.position;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        &self.input[start_pos..self.position]
    }
}

/// Abstract Syntax Tree node types
#[derive(Debug, Clone, PartialEq)]
pub enum AstNode {
    /// Numeric literal
    Literal(f64),
    
    /// Column reference
    Column(String),
    
    /// Binary operation
    BinaryOp {
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    
    /// Unary operation (for future extension)
    #[allow(dead_code)]
    UnaryOp {
        op: UnaryOperator,
        operand: Box<AstNode>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition (+)
    Add,
    
    /// Subtraction (-)
    Sub,
    
    /// Multiplication (*)
    Mul,
    
    /// Division (/)
    Div,
    
    /// Modulo (%)
    Mod,
    
    /// Power (**)
    Pow,
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::Pow => write!(f, "**"),
        }
    }
}

/// Unary operators (for future extension)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UnaryOperator {
    /// Unary negation (-)
    Neg,
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOperator::Neg => write!(f, "-"),
        }
    }
}

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Associativity {
    /// Left-associative (e.g., a - b - c = (a - b) - c)
    Left,
    
    /// Right-associative (e.g., a ** b ** c = a ** (b ** c))
    Right,
}

/// Operator metadata for precedence and associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OperatorInfo {
    /// Binding power (precedence level)
    pub binding_power: u8,
    
    /// Associativity (left or right)
    pub associativity: Associativity,
}

/// Binding power constants (precedence levels)
const BP_ADD_SUB: u8 = 10;      // + -
const BP_MUL_DIV_MOD: u8 = 20;  // * / %
const BP_POW: u8 = 30;          // **

/// Get operator information for a token
pub fn get_operator_info(token: &Token) -> Option<OperatorInfo> {
    match token {
        Token::Plus | Token::Minus => Some(OperatorInfo {
            binding_power: BP_ADD_SUB,
            associativity: Associativity::Left,
        }),
        Token::Star | Token::Slash | Token::Percent => Some(OperatorInfo {
            binding_power: BP_MUL_DIV_MOD,
            associativity: Associativity::Left,
        }),
        Token::Power => Some(OperatorInfo {
            binding_power: BP_POW,
            associativity: Associativity::Right,
        }),
        _ => None,
    }
}

/// Convert token to binary operator
fn token_to_binary_op(token: &Token) -> Option<BinaryOperator> {
    match token {
        Token::Plus => Some(BinaryOperator::Add),
        Token::Minus => Some(BinaryOperator::Sub),
        Token::Star => Some(BinaryOperator::Mul),
        Token::Slash => Some(BinaryOperator::Div),
        Token::Percent => Some(BinaryOperator::Mod),
        Token::Power => Some(BinaryOperator::Pow),
        _ => None,
    }
}

/// Pratt parser for expressions
///
/// Implements the Pratt parsing algorithm (Top-Down Operator Precedence)
/// to build an Abstract Syntax Tree from tokens.
pub struct Parser<'a> {
    /// Tokenizer for input expression
    tokenizer: Tokenizer<'a>,
    
    /// Current token being examined
    current_token: Token<'a>,
    
    /// Position of current token (for error reporting)
    current_position: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser for the given expression
    pub fn new(input: &'a str) -> Result<Self, ParseError> {
        let mut tokenizer = Tokenizer::new(input);
        let current_token = tokenizer.next_token()?;
        
        Ok(Self {
            tokenizer,
            current_token,
            current_position: 0,
        })
    }
    
    /// Advance to the next token
    fn advance(&mut self) -> Result<(), ParseError> {
        self.current_position = self.tokenizer.position;
        self.current_token = self.tokenizer.next_token()?;
        Ok(())
    }
    
    /// Parse a primary expression (number, identifier, or parenthesized expression)
    fn parse_primary(&mut self) -> Result<AstNode, ParseError> {
        match &self.current_token {
            Token::Number(n) => {
                let value = *n;
                self.advance()?;
                Ok(AstNode::Literal(value))
            }
            
            Token::Ident(name) => {
                let column_name = name.to_string();
                self.advance()?;
                Ok(AstNode::Column(column_name))
            }
            
            Token::LParen => {
                self.advance()?; // consume '('
                
                // Check for empty parentheses
                if self.current_token == Token::RParen {
                    return Err(ParseError::EmptyExpression {
                        message: "Empty parentheses are not allowed".to_string(),
                    });
                }
                
                // Parse the expression inside parentheses
                let expr = self.parse_expression(0)?;
                
                // Expect closing parenthesis
                if self.current_token != Token::RParen {
                    return Err(ParseError::UnbalancedParentheses {
                        position: self.current_position,
                        message: "Missing closing parenthesis ')'".to_string(),
                    });
                }
                
                self.advance()?; // consume ')'
                Ok(expr)
            }
            
            Token::Eof => {
                Err(ParseError::UnexpectedEof {
                    expected: "expression".to_string(),
                })
            }
            
            _ => {
                Err(ParseError::UnexpectedToken {
                    expected: "number, identifier, or '('".to_string(),
                    found: format!("{}", self.current_token),
                    position: self.current_position,
                })
            }
        }
    }
    
    /// Parse an expression with minimum binding power (Pratt algorithm)
    ///
    /// This is the core of the Pratt parsing algorithm. It handles operator
    /// precedence and associativity using binding powers.
    ///
    /// Algorithm:
    /// 1. Parse a primary expression (left side)
    /// 2. While the next operator has binding power >= min_bp:
    ///    a. Get operator info (binding power and associativity)
    ///    b. Consume the operator
    ///    c. Recursively parse right side with appropriate binding power
    ///       - For left-associative: use bp + 1 (forces left-to-right)
    ///       - For right-associative: use bp (allows right-to-left)
    ///    d. Build binary operation node
    /// 3. Return the constructed AST
    fn parse_expression(&mut self, min_bp: u8) -> Result<AstNode, ParseError> {
        // Parse the left side (primary expression)
        let mut left = self.parse_primary()?;
        
        // Process operators while they have sufficient binding power
        loop {
            // Check if current token is an operator
            let op_info = match get_operator_info(&self.current_token) {
                Some(info) => info,
                None => break, // Not an operator, we're done
            };
            
            // Check if operator has sufficient binding power
            if op_info.binding_power < min_bp {
                break;
            }
            
            // Get the operator
            let op = token_to_binary_op(&self.current_token)
                .ok_or_else(|| ParseError::UnexpectedToken {
                    expected: "operator".to_string(),
                    found: format!("{}", self.current_token),
                    position: self.current_position,
                })?;
            
            self.advance()?; // consume operator
            
            // Calculate binding power for right side based on associativity
            let right_bp = match op_info.associativity {
                Associativity::Left => op_info.binding_power + 1,
                Associativity::Right => op_info.binding_power,
            };
            
            // Parse the right side
            let right = self.parse_expression(right_bp)?;
            
            // Build binary operation node
            left = AstNode::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    /// Parse the complete expression
    ///
    /// This is the public entry point for parsing. It parses the expression
    /// and verifies that the entire input was consumed.
    pub fn parse(&mut self) -> Result<AstNode, ParseError> {
        // Check for empty expression
        if self.current_token == Token::Eof {
            return Err(ParseError::EmptyExpression {
                message: "Expression is empty or contains only whitespace".to_string(),
            });
        }
        
        // Parse the expression starting with minimum binding power 0
        let ast = self.parse_expression(0)?;
        
        // Verify we consumed the entire input
        if self.current_token != Token::Eof {
            return Err(ParseError::UnexpectedToken {
                expected: "end of expression".to_string(),
                found: format!("{}", self.current_token),
                position: self.current_position,
            });
        }
        
        Ok(ast)
    }
}

/// AST Evaluator - converts AST to Polars expressions
///
/// Validates column references and builds Polars expressions from the AST.
pub struct AstEvaluator<'a> {
    /// DataFrame for column validation
    dataframe: &'a crate::core::DataFrame,
}

impl<'a> AstEvaluator<'a> {
    /// Create a new AST evaluator
    pub fn new(dataframe: &'a crate::core::DataFrame) -> Self {
        Self { dataframe }
    }
    
    /// Validate that a column exists in the DataFrame
    fn validate_column(&self, name: &str) -> Result<(), ParseError> {
        if !self.dataframe.has_column(name) {
            return Err(ParseError::ColumnNotFound {
                column: name.to_string(),
                available: self.dataframe.column_names(),
            });
        }
        Ok(())
    }
    
    /// Evaluate an AST node to a Polars expression
    ///
    /// Recursively traverses the AST and builds the corresponding Polars expression.
    /// Validates column references during evaluation.
    pub fn evaluate(&self, node: &AstNode) -> Result<polars::prelude::Expr, ParseError> {
        use polars::prelude::*;
        
        match node {
            AstNode::Literal(value) => {
                Ok(lit(*value))
            }
            
            AstNode::Column(name) => {
                self.validate_column(name)?;
                Ok(col(name))
            }
            
            AstNode::BinaryOp { op, left, right } => {
                let left_expr = self.evaluate(left)?;
                let right_expr = self.evaluate(right)?;
                
                let result = match op {
                    BinaryOperator::Add => left_expr + right_expr,
                    BinaryOperator::Sub => left_expr - right_expr,
                    BinaryOperator::Mul => left_expr * right_expr,
                    BinaryOperator::Div => left_expr / right_expr,
                    BinaryOperator::Mod => left_expr % right_expr,
                    BinaryOperator::Pow => left_expr.pow(right_expr),
                };
                
                Ok(result)
            }
            
            AstNode::UnaryOp { op, operand } => {
                let operand_expr = self.evaluate(operand)?;
                
                let result = match op {
                    UnaryOperator::Neg => -operand_expr,
                };
                
                Ok(result)
            }
        }
    }
}

/// Parse expression string into Polars expression using the new parser
///
/// This is the main entry point for parsing expressions. It:
/// 1. Tokenizes the expression
/// 2. Parses it into an AST
/// 3. Evaluates the AST to a Polars expression
/// 4. Validates column references
///
/// # Arguments
///
/// * `expression` - Expression string to parse
/// * `df` - DataFrame for column validation
///
/// # Returns
///
/// * `Result<Expr, ParseError>` - Polars expression or parse error
///
/// # Example
///
/// ```rust,ignore
/// let expr = parse_expression_new("weight / (height ** 2)", &df)?;
/// ```
pub fn parse_expression_new(
    expression: &str,
    df: &crate::core::DataFrame,
) -> Result<polars::prelude::Expr, ParseError> {
    // Parse the expression into an AST
    let mut parser = Parser::new(expression)?;
    let ast = parser.parse()?;
    
    // Evaluate the AST to a Polars expression
    let evaluator = AstEvaluator::new(df);
    evaluator.evaluate(&ast)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_to_additory_invalid_character() {
        let expr = "price + @";
        let error = ParseError::InvalidCharacter { ch: '@', position: 8 };
        let additory_error = error.to_additory_error(expr);
        
        match additory_error {
            AdditoryError::Operation(msg, suggestion) => {
                assert!(msg.contains("Invalid character '@'"));
                assert!(msg.contains("position 8"));
                assert!(suggestion.contains("Valid characters"));
            }
            _ => panic!("Expected OperationFailed error"),
        }
    }

    #[test]
    fn test_error_to_additory_unbalanced_parens() {
        let expr = "price * (quantity + 5";
        let error = ParseError::UnbalancedParentheses {
            position: 8,
            message: "Missing closing parenthesis ')'".to_string(),
        };
        let additory_error = error.to_additory_error(expr);
        
        match additory_error {
            AdditoryError::Operation(msg, suggestion) => {
                assert!(msg.contains("Missing closing parenthesis"));
                assert!(suggestion.contains("matching closing parentheses"));
            }
            _ => panic!("Expected OperationFailed error"),
        }
    }

    #[test]
    fn test_error_to_additory_column_not_found() {
        let expr = "nonexistent + 5";
        let error = ParseError::ColumnNotFound {
            column: "nonexistent".to_string(),
            available: vec!["price".to_string(), "quantity".to_string()],
        };
        let additory_error = error.to_additory_error(expr);
        
        match additory_error {
            AdditoryError::Operation(msg, suggestion) => {
                assert!(msg.contains("Column 'nonexistent' not found"));
                assert!(suggestion.contains("price"));
                assert!(suggestion.contains("quantity"));
            }
            _ => panic!("Expected OperationFailed error"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = ParseError::InvalidCharacter { ch: '@', position: 5 };
        assert_eq!(error.to_string(), "Invalid character '@' at position 5");
        
        let error = ParseError::EmptyExpression {
            message: "Expression is empty".to_string(),
        };
        assert_eq!(error.to_string(), "Expression is empty");
    }
    
    // Tokenizer tests
    
    #[test]
    fn test_tokenize_simple_expression() {
        let mut tokenizer = Tokenizer::new("a + b");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("a"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Plus);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("b"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_power_operator() {
        let mut tokenizer = Tokenizer::new("a ** 2");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("a"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Power);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Number(2.0));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_power_vs_multiply() {
        // Test that ** is recognized as power, not two multiplications
        let mut tokenizer = Tokenizer::new("a * b ** c");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("a"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Star);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("b"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Power);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("c"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_parentheses() {
        let mut tokenizer = Tokenizer::new("(a + b) * c");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::LParen);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("a"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Plus);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("b"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::RParen);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Star);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("c"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_numbers() {
        let mut tokenizer = Tokenizer::new("42 3.14 1.5e-10");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Number(42.0));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Number(3.14));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Number(1.5e-10));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_all_operators() {
        let mut tokenizer = Tokenizer::new("+ - * / % **");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Plus);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Minus);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Star);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Slash);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Percent);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Power);
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_whitespace_invariance() {
        let expr1 = "a+b";
        let expr2 = "a + b";
        let expr3 = "a  +  b";
        
        let tokens1: Vec<_> = {
            let mut t = Tokenizer::new(expr1);
            let mut tokens = Vec::new();
            loop {
                let token = t.next_token().unwrap();
                if token == Token::Eof {
                    break;
                }
                tokens.push(token);
            }
            tokens
        };
        
        let tokens2: Vec<_> = {
            let mut t = Tokenizer::new(expr2);
            let mut tokens = Vec::new();
            loop {
                let token = t.next_token().unwrap();
                if token == Token::Eof {
                    break;
                }
                tokens.push(token);
            }
            tokens
        };
        
        let tokens3: Vec<_> = {
            let mut t = Tokenizer::new(expr3);
            let mut tokens = Vec::new();
            loop {
                let token = t.next_token().unwrap();
                if token == Token::Eof {
                    break;
                }
                tokens.push(token);
            }
            tokens
        };
        
        assert_eq!(tokens1, tokens2);
        assert_eq!(tokens2, tokens3);
    }
    
    #[test]
    fn test_tokenize_invalid_character() {
        let mut tokenizer = Tokenizer::new("a + @");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("a"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Plus);
        
        let result = tokenizer.next_token();
        assert!(result.is_err());
        
        match result {
            Err(ParseError::InvalidCharacter { ch, position }) => {
                assert_eq!(ch, '@');
                assert_eq!(position, 4);
            }
            _ => panic!("Expected InvalidCharacter error"),
        }
    }
    
    #[test]
    fn test_tokenize_empty_input() {
        let mut tokenizer = Tokenizer::new("");
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }
    
    #[test]
    fn test_tokenize_identifiers_with_underscores() {
        let mut tokenizer = Tokenizer::new("price_usd total_amount");
        
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("price_usd"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Ident("total_amount"));
        assert_eq!(tokenizer.next_token().unwrap(), Token::Eof);
    }

    
    // Parser tests
    
    #[test]
    fn test_parse_simple_addition() {
        let mut parser = Parser::new("a + b").unwrap();
        let ast = parser.parse().unwrap();
        
        match ast {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(op, BinaryOperator::Add);
                assert_eq!(*left, AstNode::Column("a".to_string()));
                assert_eq!(*right, AstNode::Column("b".to_string()));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
    
    #[test]
    fn test_parse_power_operator() {
        let mut parser = Parser::new("height ** 2").unwrap();
        let ast = parser.parse().unwrap();
        
        match ast {
            AstNode::BinaryOp { op, left, right } => {
                assert_eq!(op, BinaryOperator::Pow);
                assert_eq!(*left, AstNode::Column("height".to_string()));
                assert_eq!(*right, AstNode::Literal(2.0));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
    
    #[test]
    fn test_parse_parentheses() {
        let mut parser = Parser::new("(a + b) * c").unwrap();
        let ast = parser.parse().unwrap();
        
        match ast {
            AstNode::BinaryOp { op: op_outer, left, right } => {
                assert_eq!(op_outer, BinaryOperator::Mul);
                
                // Left should be (a + b)
                match *left {
                    AstNode::BinaryOp { op, left: inner_left, right: inner_right } => {
                        assert_eq!(op, BinaryOperator::Add);
                        assert_eq!(*inner_left, AstNode::Column("a".to_string()));
                        assert_eq!(*inner_right, AstNode::Column("b".to_string()));
                    }
                    _ => panic!("Expected BinaryOp for left side"),
                }
                
                // Right should be c
                assert_eq!(*right, AstNode::Column("c".to_string()));
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
    
    #[test]
    fn test_parse_precedence() {
        // 2 + 3 * 4 should be 2 + (3 * 4)
        let mut parser = Parser::new("2 + 3 * 4").unwrap();
        let ast = parser.parse().unwrap();
        
        match ast {
            AstNode::BinaryOp { op: op_outer, left, right } => {
                assert_eq!(op_outer, BinaryOperator::Add);
                assert_eq!(*left, AstNode::Literal(2.0));
                
                // Right should be (3 * 4)
                match *right {
                    AstNode::BinaryOp { op, left: inner_left, right: inner_right } => {
                        assert_eq!(op, BinaryOperator::Mul);
                        assert_eq!(*inner_left, AstNode::Literal(3.0));
                        assert_eq!(*inner_right, AstNode::Literal(4.0));
                    }
                    _ => panic!("Expected BinaryOp for right side"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
    
    #[test]
    fn test_parse_power_right_associative() {
        // 2 ** 3 ** 2 should be 2 ** (3 ** 2)
        let mut parser = Parser::new("2 ** 3 ** 2").unwrap();
        let ast = parser.parse().unwrap();
        
        match ast {
            AstNode::BinaryOp { op: op_outer, left, right } => {
                assert_eq!(op_outer, BinaryOperator::Pow);
                assert_eq!(*left, AstNode::Literal(2.0));
                
                // Right should be (3 ** 2)
                match *right {
                    AstNode::BinaryOp { op, left: inner_left, right: inner_right } => {
                        assert_eq!(op, BinaryOperator::Pow);
                        assert_eq!(*inner_left, AstNode::Literal(3.0));
                        assert_eq!(*inner_right, AstNode::Literal(2.0));
                    }
                    _ => panic!("Expected BinaryOp for right side"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
    
    #[test]
    fn test_parse_empty_expression() {
        let result = Parser::new("");
        assert!(result.is_ok());
        
        let mut parser = result.unwrap();
        let result = parser.parse();
        assert!(result.is_err());
        
        match result {
            Err(ParseError::EmptyExpression { .. }) => {}
            _ => panic!("Expected EmptyExpression error"),
        }
    }
    
    #[test]
    fn test_parse_empty_parentheses() {
        let mut parser = Parser::new("()").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
        
        match result {
            Err(ParseError::EmptyExpression { .. }) => {}
            _ => panic!("Expected EmptyExpression error"),
        }
    }
    
    #[test]
    fn test_parse_unbalanced_parentheses() {
        let mut parser = Parser::new("(a + b").unwrap();
        let result = parser.parse();
        assert!(result.is_err());
        
        match result {
            Err(ParseError::UnbalancedParentheses { .. }) => {}
            _ => panic!("Expected UnbalancedParentheses error"),
        }
    }
    
    #[test]
    fn test_parse_nested_parentheses() {
        let mut parser = Parser::new("((a + b) * c)").unwrap();
        let ast = parser.parse().unwrap();
        
        // Should parse successfully
        match ast {
            AstNode::BinaryOp { op, .. } => {
                assert_eq!(op, BinaryOperator::Mul);
            }
            _ => panic!("Expected BinaryOp"),
        }
    }
    
    #[test]
    fn test_parse_complex_expression() {
        // weight / (height ** 2)
        let mut parser = Parser::new("weight / (height ** 2)").unwrap();
        let ast = parser.parse().unwrap();
        
        match ast {
            AstNode::BinaryOp { op: op_outer, left, right } => {
                assert_eq!(op_outer, BinaryOperator::Div);
                assert_eq!(*left, AstNode::Column("weight".to_string()));
                
                // Right should be (height ** 2)
                match *right {
                    AstNode::BinaryOp { op, left: inner_left, right: inner_right } => {
                        assert_eq!(op, BinaryOperator::Pow);
                        assert_eq!(*inner_left, AstNode::Column("height".to_string()));
                        assert_eq!(*inner_right, AstNode::Literal(2.0));
                    }
                    _ => panic!("Expected BinaryOp for right side"),
                }
            }
            _ => panic!("Expected BinaryOp"),
        }
    }

    
    // AST Evaluator tests
    
    fn create_test_df() -> crate::core::DataFrame {
        use polars::prelude::*;
        let polars_df = df! {
            "price" => &[10.0, 20.0, 30.0],
            "quantity" => &[2, 3, 4],
            "height" => &[1.75, 1.80, 1.65],
            "weight" => &[70.0, 80.0, 60.0],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }
    
    #[test]
    fn test_evaluate_simple_addition() {
        let df = create_test_df();
        let expr = parse_expression_new("price + 5", &df).unwrap();
        
        // Apply expression to DataFrame
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("result")])
            .collect()
            .unwrap();
        
        let col = result.column("result").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(values, vec![15.0, 25.0, 35.0]);
    }
    
    #[test]
    fn test_evaluate_power_operator() {
        let df = create_test_df();
        let expr = parse_expression_new("height ** 2", &df).unwrap();
        
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("result")])
            .collect()
            .unwrap();
        
        let col = result.column("result").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        
        // height ** 2: 1.75^2 = 3.0625, 1.80^2 = 3.24, 1.65^2 = 2.7225
        assert!((values[0] - 3.0625).abs() < 0.0001);
        assert!((values[1] - 3.24).abs() < 0.0001);
        assert!((values[2] - 2.7225).abs() < 0.0001);
    }
    
    #[test]
    fn test_evaluate_bmi_calculation() {
        let df = create_test_df();
        let expr = parse_expression_new("weight / (height ** 2)", &df).unwrap();
        
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("bmi")])
            .collect()
            .unwrap();
        
        let col = result.column("bmi").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        
        // BMI = weight / height^2
        // 70 / 3.0625 ≈ 22.86
        // 80 / 3.24 ≈ 24.69
        // 60 / 2.7225 ≈ 22.04
        assert!((values[0] - 22.86).abs() < 0.01);
        assert!((values[1] - 24.69).abs() < 0.01);
        assert!((values[2] - 22.04).abs() < 0.01);
    }
    
    #[test]
    fn test_evaluate_precedence() {
        let df = create_test_df();
        // price + 2 * 3 should be price + (2 * 3) = price + 6
        let expr = parse_expression_new("price + 2 * 3", &df).unwrap();
        
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("result")])
            .collect()
            .unwrap();
        
        let col = result.column("result").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        
        // price + 6: 10+6=16, 20+6=26, 30+6=36
        assert_eq!(values, vec![16.0, 26.0, 36.0]);
    }
    
    #[test]
    fn test_evaluate_column_not_found() {
        let df = create_test_df();
        let result = parse_expression_new("nonexistent + 5", &df);
        
        assert!(result.is_err());
        match result {
            Err(ParseError::ColumnNotFound { column, available }) => {
                assert_eq!(column, "nonexistent");
                assert!(available.contains(&"price".to_string()));
                assert!(available.contains(&"quantity".to_string()));
            }
            _ => panic!("Expected ColumnNotFound error"),
        }
    }
    
    #[test]
    fn test_evaluate_parentheses_override_precedence() {
        let df = create_test_df();
        // (price + 2) * 3 should be (price + 2) * 3
        let expr = parse_expression_new("(price + 2) * 3", &df).unwrap();
        
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("result")])
            .collect()
            .unwrap();
        
        let col = result.column("result").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        
        // (10+2)*3=36, (20+2)*3=66, (30+2)*3=96
        assert_eq!(values, vec![36.0, 66.0, 96.0]);
    }
    
    #[test]
    fn test_evaluate_power_right_associative() {
        let df = create_test_df();
        // quantity ** 2 ** 2 should be quantity ** (2 ** 2) = quantity ** 4
        let expr = parse_expression_new("quantity ** 2 ** 2", &df).unwrap();
        
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("result")])
            .collect()
            .unwrap();
        
        let col = result.column("result").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        
        // 2^4=16, 3^4=81, 4^4=256
        assert_eq!(values, vec![16.0, 81.0, 256.0]);
    }
    
    #[test]
    fn test_evaluate_power_with_parentheses() {
        let df = create_test_df();
        // (quantity ** 2) ** 2 should be quantity^2 then squared
        let expr = parse_expression_new("(quantity ** 2) ** 2", &df).unwrap();
        
        use polars::prelude::*;
        let result = df.inner().clone()
            .lazy()
            .select([expr.alias("result")])
            .collect()
            .unwrap();
        
        let col = result.column("result").unwrap();
        let values: Vec<f64> = col.f64().unwrap().into_iter().map(|v| v.unwrap()).collect();
        
        // (2^2)^2=16, (3^2)^2=81, (4^2)^2=256
        assert_eq!(values, vec![16.0, 81.0, 256.0]);
    }
}
