use std::fmt;

/// Helper enum for predicate or path
enum PredicateOrPath {
    Predicate(PatternElement),
    Path(PropertyPath),
}

/// Parsed SPARQL query representation
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedQuery {
    pub select_vars: Vec<String>,
    pub where_patterns: Vec<TriplePattern>,
    pub filters: Vec<FilterExpression>,
    pub optional: Vec<TriplePattern>,
    pub unions: Vec<Vec<TriplePattern>>,
    pub group_by: Vec<String>,
    pub aggregations: Vec<Aggregation>,
}

/// Triple pattern in WHERE clause
#[derive(Debug, Clone, PartialEq)]
pub enum TriplePattern {
    Simple {
        subject: PatternElement,
        predicate: PatternElement,
        object: PatternElement,
    },
    PropertyPath {
        subject: PatternElement,
        path: PropertyPath,
        object: PatternElement,
    },
}

/// Element in a triple pattern (variable or constant)
#[derive(Debug, Clone, PartialEq)]
pub enum PatternElement {
    Variable(String),  // ?x
    Constant(String),  // cnv:Verb or "literal"
}

/// Property path expressions
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyPath {
    Direct(String),                           // :hasMember
    Inverse(Box<PropertyPath>),               // ^:hasMember
    Sequence(Box<PropertyPath>, Box<PropertyPath>), // :p1/:p2
    Alternative(Box<PropertyPath>, Box<PropertyPath>), // :p1|:p2
    ZeroOrMore(Box<PropertyPath>),            // :p*
    OneOrMore(Box<PropertyPath>),             // :p+
    ZeroOrOne(Box<PropertyPath>),             // :p?
}

/// Filter expressions
#[derive(Debug, Clone, PartialEq)]
pub enum FilterExpression {
    Contains { var: String, literal: String },
    StartsWith { var: String, literal: String },
    EndsWith { var: String, literal: String },
    GreaterThan { var: String, value: String },
    LessThan { var: String, value: String },
    Equals { var: String, value: String },
    NotEquals { var: String, value: String },
    And { left: Box<FilterExpression>, right: Box<FilterExpression> },
    Or { left: Box<FilterExpression>, right: Box<FilterExpression> },
    Not { expr: Box<FilterExpression> },
}

/// Aggregation functions
#[derive(Debug, Clone, PartialEq)]
pub enum Aggregation {
    Count { var: String, distinct: bool, alias: String },
    Sum { var: String, alias: String },
    Min { var: String, alias: String },
    Max { var: String, alias: String },
    Avg { var: String, alias: String },
}

/// Parse errors
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken { expected: String, found: String, position: usize },
    UnexpectedEndOfInput { expected: String },
    InvalidSyntax { message: String, position: usize },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, position } => {
                write!(f, "Parse error at position {}: expected {}, found {}", position, expected, found)
            }
            ParseError::UnexpectedEndOfInput { expected } => {
                write!(f, "Parse error: unexpected end of input, expected {}", expected)
            }
            ParseError::InvalidSyntax { message, position } => {
                write!(f, "Parse error at position {}: {}", position, message)
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// SPARQL parser using recursive descent
pub struct SparqlParser {
    tokens: Vec<String>,
    pos: usize,
}

impl SparqlParser {
    /// Parse a SPARQL query string
    pub fn parse(sparql: &str) -> Result<ParsedQuery, ParseError> {
        let tokens = Self::tokenize(sparql);
        let mut parser = SparqlParser { tokens, pos: 0 };
        parser.parse_query()
    }

    /// Tokenize SPARQL query
    fn tokenize(sparql: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        let mut in_string = false;
        let mut in_angle = false;

        for ch in sparql.chars() {
            match ch {
                '"' if !in_angle => {
                    in_string = !in_string;
                    current.push(ch);
                }
                '<' if !in_string => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    in_angle = true;
                    current.push(ch);
                }
                '>' if in_angle && !in_string => {
                    current.push(ch);
                    tokens.push(current.clone());
                    current.clear();
                    in_angle = false;
                }
                ' ' | '\t' | '\n' | '\r' if !in_string && !in_angle => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                }
                '(' | ')' | '{' | '}' | '.' | ';' | ',' if !in_string && !in_angle => {
                    if !current.is_empty() {
                        tokens.push(current.clone());
                        current.clear();
                    }
                    tokens.push(ch.to_string());
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty() {
            tokens.push(current);
        }

        tokens
    }

    /// Parse complete query
    fn parse_query(&mut self) -> Result<ParsedQuery, ParseError> {
        let (select_vars, aggregations) = self.parse_select()?;
        self.expect_keyword("WHERE")?;
        self.expect_token("{")?;

        let mut where_patterns = Vec::new();
        let mut filters = Vec::new();
        let mut optional = Vec::new();
        let mut unions = Vec::new();

        while !self.check_token("}") {
            if self.check_keyword("OPTIONAL") {
                self.advance();
                self.expect_token("{")?;
                let opt_patterns = self.parse_triple_patterns()?;
                optional.extend(opt_patterns);
                self.expect_token("}")?;
            } else if self.check_keyword("UNION") {
                // Parse UNION blocks
                let union_patterns = self.parse_union_blocks()?;
                unions.push(union_patterns);
            } else if self.check_keyword("FILTER") {
                self.advance();
                let filter = self.parse_filter()?;
                filters.push(filter);
            } else {
                let pattern = self.parse_triple_pattern()?;
                where_patterns.push(pattern);
                if self.check_token(".") {
                    self.advance();
                }
            }
        }

        self.expect_token("}")?;

        let group_by = if self.check_keyword("GROUP") {
            self.parse_group_by()?
        } else {
            Vec::new()
        };

        Ok(ParsedQuery {
            select_vars,
            where_patterns,
            filters,
            optional,
            unions,
            group_by,
            aggregations,
        })
    }

    /// Parse SELECT clause
    fn parse_select(&mut self) -> Result<(Vec<String>, Vec<Aggregation>), ParseError> {
        self.expect_keyword("SELECT")?;

        let mut vars = Vec::new();
        let mut aggregations = Vec::new();

        while !self.check_keyword("WHERE") {
            if self.check_token("(") {
                // Aggregation: (COUNT(?var) AS ?alias)
                self.advance();
                let agg = self.parse_aggregation()?;
                aggregations.push(agg.clone());
                // Extract alias for select vars
                let alias = match &agg {
                    Aggregation::Count { alias, .. } |
                    Aggregation::Sum { alias, .. } |
                    Aggregation::Min { alias, .. } |
                    Aggregation::Max { alias, .. } |
                    Aggregation::Avg { alias, .. } => alias.clone(),
                };
                vars.push(alias);
            } else if let Some(token) = self.current_token() {
                if token.starts_with('?') {
                    vars.push(token.clone());
                    self.advance();
                } else if token == "*" {
                    vars.push("*".to_string());
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok((vars, aggregations))
    }

    /// Parse aggregation function
    fn parse_aggregation(&mut self) -> Result<Aggregation, ParseError> {
        let func = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "aggregation function".to_string() })?
            .to_uppercase();

        self.advance();
        self.expect_token("(")?;

        let distinct = if self.check_keyword("DISTINCT") {
            self.advance();
            true
        } else {
            false
        };

        let var = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "variable".to_string() })?
            .clone();
        self.advance();

        self.expect_token(")")?;
        self.expect_keyword("AS")?;

        let alias = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "alias".to_string() })?
            .clone();
        self.advance();

        self.expect_token(")")?;

        match func.as_str() {
            "COUNT" => Ok(Aggregation::Count { var, distinct, alias }),
            "SUM" => Ok(Aggregation::Sum { var, alias }),
            "MIN" => Ok(Aggregation::Min { var, alias }),
            "MAX" => Ok(Aggregation::Max { var, alias }),
            "AVG" => Ok(Aggregation::Avg { var, alias }),
            _ => Err(ParseError::InvalidSyntax {
                message: format!("Unknown aggregation function: {}", func),
                position: self.pos,
            }),
        }
    }

    /// Parse triple patterns
    fn parse_triple_patterns(&mut self) -> Result<Vec<TriplePattern>, ParseError> {
        let mut patterns = Vec::new();
        while !self.check_token("}") {
            if self.check_keyword("FILTER") || self.check_keyword("OPTIONAL") || self.check_keyword("UNION") {
                break;
            }
            let pattern = self.parse_triple_pattern()?;
            patterns.push(pattern);
            if self.check_token(".") {
                self.advance();
            }
        }
        Ok(patterns)
    }

    /// Parse single triple pattern
    fn parse_triple_pattern(&mut self) -> Result<TriplePattern, ParseError> {
        let subject = self.parse_pattern_element()?;
        let predicate_or_path = self.parse_predicate_or_path()?;
        let object = self.parse_pattern_element()?;

        match predicate_or_path {
            PredicateOrPath::Predicate(pred) => {
                Ok(TriplePattern::Simple {
                    subject,
                    predicate: pred,
                    object,
                })
            }
            PredicateOrPath::Path(path) => {
                Ok(TriplePattern::PropertyPath {
                    subject,
                    path,
                    object,
                })
            }
        }
    }

    /// Parse pattern element (variable or constant)
    fn parse_pattern_element(&mut self) -> Result<PatternElement, ParseError> {
        let token = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "pattern element".to_string() })?
            .clone();

        self.advance();

        if token.starts_with('?') {
            Ok(PatternElement::Variable(token))
        } else {
            Ok(PatternElement::Constant(token))
        }
    }

    /// Parse predicate or property path
    fn parse_predicate_or_path(&mut self) -> Result<PredicateOrPath, ParseError> {
        let token = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "predicate".to_string() })?
            .clone();

        // Check for property path modifiers
        let next_token = self.peek_token(1);

        if let Some(next) = next_token {
            if next == "*" || next == "+" || next == "?" {
                // Property path
                self.advance();
                let modifier = self.current_token().unwrap().clone();
                self.advance();

                let base_path = PropertyPath::Direct(token);
                let path = match modifier.as_str() {
                    "*" => PropertyPath::ZeroOrMore(Box::new(base_path)),
                    "+" => PropertyPath::OneOrMore(Box::new(base_path)),
                    "?" => PropertyPath::ZeroOrOne(Box::new(base_path)),
                    _ => unreachable!(),
                };

                return Ok(PredicateOrPath::Path(path));
            }
        }

        // Check for inverse property path (^)
        if token.starts_with('^') {
            self.advance();
            let path_name = token[1..].to_string();
            let path = PropertyPath::Inverse(Box::new(PropertyPath::Direct(path_name)));
            return Ok(PredicateOrPath::Path(path));
        }

        // Simple predicate
        self.advance();
        if token.starts_with('?') {
            Ok(PredicateOrPath::Predicate(PatternElement::Variable(token)))
        } else {
            Ok(PredicateOrPath::Predicate(PatternElement::Constant(token)))
        }
    }

    /// Parse FILTER expression
    fn parse_filter(&mut self) -> Result<FilterExpression, ParseError> {
        self.expect_token("(")?;

        let expr = if self.check_keyword("CONTAINS") {
            self.advance();
            self.expect_token("(")?;
            let var = self.current_token()
                .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "variable".to_string() })?
                .clone();
            self.advance();
            self.expect_token(",")?;
            let literal = self.parse_string_literal()?;
            self.expect_token(")")?;
            FilterExpression::Contains { var, literal }
        } else if self.check_keyword("STRSTARTS") {
            self.advance();
            self.expect_token("(")?;
            let var = self.current_token()
                .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "variable".to_string() })?
                .clone();
            self.advance();
            self.expect_token(",")?;
            let literal = self.parse_string_literal()?;
            self.expect_token(")")?;
            FilterExpression::StartsWith { var, literal }
        } else if self.check_keyword("STRENDS") {
            self.advance();
            self.expect_token("(")?;
            let var = self.current_token()
                .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "variable".to_string() })?
                .clone();
            self.advance();
            self.expect_token(",")?;
            let literal = self.parse_string_literal()?;
            self.expect_token(")")?;
            FilterExpression::EndsWith { var, literal }
        } else {
            // Comparison or logical expression
            self.parse_comparison_or_logical()?
        };

        self.expect_token(")")?;
        Ok(expr)
    }

    /// Parse comparison or logical expression
    fn parse_comparison_or_logical(&mut self) -> Result<FilterExpression, ParseError> {
        let var = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "variable".to_string() })?
            .clone();
        self.advance();

        let op = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "operator".to_string() })?
            .clone();
        self.advance();

        let value = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "value".to_string() })?
            .clone();
        self.advance();

        match op.as_str() {
            "=" => Ok(FilterExpression::Equals { var, value }),
            "!=" => Ok(FilterExpression::NotEquals { var, value }),
            ">" => Ok(FilterExpression::GreaterThan { var, value }),
            "<" => Ok(FilterExpression::LessThan { var, value }),
            _ => Err(ParseError::InvalidSyntax {
                message: format!("Unknown operator: {}", op),
                position: self.pos,
            }),
        }
    }

    /// Parse string literal
    fn parse_string_literal(&mut self) -> Result<String, ParseError> {
        let token = self.current_token()
            .ok_or_else(|| ParseError::UnexpectedEndOfInput { expected: "string literal".to_string() })?
            .clone();
        self.advance();

        if token.starts_with('"') && token.ends_with('"') {
            Ok(token[1..token.len()-1].to_string())
        } else {
            Ok(token)
        }
    }

    /// Parse UNION blocks
    fn parse_union_blocks(&mut self) -> Result<Vec<TriplePattern>, ParseError> {
        let mut patterns = Vec::new();

        // First block before UNION
        self.expect_token("{")?;
        patterns.extend(self.parse_triple_patterns()?);
        self.expect_token("}")?;

        // UNION blocks
        while self.check_keyword("UNION") {
            self.advance();
            self.expect_token("{")?;
            patterns.extend(self.parse_triple_patterns()?);
            self.expect_token("}")?;
        }

        Ok(patterns)
    }

    /// Parse GROUP BY clause
    fn parse_group_by(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect_keyword("GROUP")?;
        self.expect_keyword("BY")?;

        let mut vars = Vec::new();
        while let Some(token) = self.current_token() {
            if token.starts_with('?') {
                vars.push(token.clone());
                self.advance();
            } else {
                break;
            }
        }

        Ok(vars)
    }

    // Helper methods

    fn current_token(&self) -> Option<&String> {
        self.tokens.get(self.pos)
    }

    fn peek_token(&self, offset: usize) -> Option<&String> {
        self.tokens.get(self.pos + offset)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn check_token(&self, expected: &str) -> bool {
        self.current_token().map_or(false, |t| t == expected)
    }

    fn check_keyword(&self, expected: &str) -> bool {
        self.current_token().map_or(false, |t| t.eq_ignore_ascii_case(expected))
    }

    fn expect_token(&mut self, expected: &str) -> Result<(), ParseError> {
        if let Some(token) = self.current_token() {
            if token == expected {
                self.advance();
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken {
                    expected: expected.to_string(),
                    found: token.clone(),
                    position: self.pos,
                })
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput {
                expected: expected.to_string(),
            })
        }
    }

    fn expect_keyword(&mut self, expected: &str) -> Result<(), ParseError> {
        if let Some(token) = self.current_token() {
            if token.eq_ignore_ascii_case(expected) {
                self.advance();
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken {
                    expected: expected.to_string(),
                    found: token.clone(),
                    position: self.pos,
                })
            }
        } else {
            Err(ParseError::UnexpectedEndOfInput {
                expected: expected.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_triple_pattern() {
        let query = "SELECT ?verb WHERE { ?verb a cnv:Verb }";
        let parsed = SparqlParser::parse(query).unwrap();

        assert_eq!(parsed.select_vars, vec!["?verb"]);
        assert_eq!(parsed.where_patterns.len(), 1);
        assert_eq!(parsed.filters.len(), 0);
    }

    #[test]
    fn test_filter_contains() {
        let query = r#"SELECT ?verb WHERE { ?verb a cnv:Verb . ?verb rdfs:comment ?comment . FILTER(CONTAINS(?comment, "status")) }"#;
        let parsed = SparqlParser::parse(query).unwrap();

        assert_eq!(parsed.select_vars, vec!["?verb"]);
        assert_eq!(parsed.where_patterns.len(), 2);
        assert_eq!(parsed.filters.len(), 1);

        match &parsed.filters[0] {
            FilterExpression::Contains { var, literal } => {
                assert_eq!(var, "?comment");
                assert_eq!(literal, "status");
            }
            _ => panic!("Expected Contains filter"),
        }
    }

    #[test]
    fn test_property_path_kleene_star() {
        let query = "SELECT ?x WHERE { ?x :hasMember* ?y }";
        let parsed = SparqlParser::parse(query).unwrap();

        assert_eq!(parsed.select_vars, vec!["?x"]);
        assert_eq!(parsed.where_patterns.len(), 1);

        match &parsed.where_patterns[0] {
            TriplePattern::PropertyPath { path, .. } => {
                match path {
                    PropertyPath::ZeroOrMore(_) => {},
                    _ => panic!("Expected ZeroOrMore path"),
                }
            }
            _ => panic!("Expected PropertyPath pattern"),
        }
    }

    #[test]
    fn test_aggregation_count() {
        let query = "SELECT (COUNT(?verb) AS ?count) WHERE { ?verb a cnv:Verb }";
        let parsed = SparqlParser::parse(query).unwrap();

        assert_eq!(parsed.select_vars, vec!["?count"]);
        assert_eq!(parsed.aggregations.len(), 1);

        match &parsed.aggregations[0] {
            Aggregation::Count { var, distinct, alias } => {
                assert_eq!(var, "?verb");
                assert!(!distinct);
                assert_eq!(alias, "?count");
            }
            _ => panic!("Expected Count aggregation"),
        }
    }
}
