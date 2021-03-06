use lexer;
use lexer::LexItem;

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum GrammarItem {
    Query,
    LogFile { fields: Vec<String>, filename: String },
    Condition { field: String, mode: WhereComparator, value: String },
    Limit { number_of_rows: usize, direction: LimitDirection },
    LogResult
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum LimitDirection {
    First,
    Last
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
pub enum WhereComparator {
    StrictEquals,
    Like
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ASTNode {
    pub left: Option<Box<ASTNode>>,
    pub right: Option<Box<ASTNode>>,
    pub entry: GrammarItem,
}

impl ASTNode {
    pub fn new(entry: GrammarItem, left: Option<Box<ASTNode>>, right: Option<Box<ASTNode>>) -> ASTNode {
        ASTNode {
            entry: entry,
            left: left,
            right: right
        }
    }
}

pub struct Parser {
    query: String,
    token_index: usize,
    token_stream: Vec<LexItem>
}

impl Parser {
    pub fn new(query: String) -> Parser {
        Parser {
            query: query,
            token_stream: vec!(),
            token_index: 0
        }
    }

    fn current_token(&self) -> Option<&lexer::LexItem> {
        if self.token_index < self.token_stream.len() {
            Some(&self.token_stream[self.token_index])
        } else {
            None
        }
    }

    fn next_token(&self) -> Option<&lexer::LexItem> {
        if self.token_index + 1 < self.token_stream.len() {
            Some(&self.token_stream[self.token_index + 1])
        } else {
            None
        }
    }

    fn consume_token(&mut self) {
        self.token_index += 1;
    }

    fn expect_eof(&self) -> Result<(), String> {
        if let Some(current_token) = self.current_token() {
            match *current_token {
                lexer::LexItem::EOF => {
                    Ok(())
                },
                _ => {
                    Err(format!("Expected EOF, got {:?}", self.current_token()))
                }
            }
        } else {
            Err(format!("Expected EOF, got {:?}", self.current_token()))
        }
    }

    fn expect_equals(&self) -> Result<(), String> {
        if let Some(current_token) = self.current_token() {
            match *current_token {
                lexer::LexItem::Equals => {
                    Ok(())
                },
                _ => {
                    Err(format!("Expected assign, got {:?}", self.current_token()))
                }
            }
        } else {
            Err("Expected assign, got EOF".into())
        }
    }

    fn expect_identifier(&self, identifier_value: Option<&str>) -> Result<String, String> {
        match self.current_token() {
            Some(&lexer::LexItem::Identifier(ref identifier)) => {
                if identifier_value.is_some() {
                    if identifier_value.unwrap() == identifier {
                        Ok(identifier.clone())
                    } else {
                        Err(format!("Expected Identifier {:?}, got {:?}", identifier_value.unwrap(), self.current_token()))
                    }
                } else {
                    Ok(identifier.clone())
                }
            },
            _ => { Err(format!("Expected Identifier, got {:?}", self.current_token())) }
        }
    }

    fn expect_number(&self, expected_number: Option<usize>) -> Result<usize, String> {
        match self.current_token() {
            Some(&lexer::LexItem::Number(ref num)) => {
                if expected_number.is_some() {
                    if &expected_number.unwrap() == num {
                        Ok(num.clone())
                    } else {
                        Err(format!("Expected Number {:?}, got {:?}", expected_number.unwrap(), self.current_token()))
                    }
                } else {
                    Ok(num.clone())
                }
            },
            _ => { Err(format!("Expected Number, got {:?}", self.current_token())) }
        }
    }

    fn parse_log_file_where_value(&self) -> Result<String, String> {
        if let Some(&lexer::LexItem::Str(ref s)) = self.current_token() {
            Ok(s.clone())
        } else {
            Err(format!("Expected String, got {:?}", self.current_token()))
        }
    }

    fn parse_log_file(&mut self) -> Result<ASTNode, String> {
        let log_file_name;
        let log_file_fields = try!(self.expect_select_field_list());
        self.consume_token();

        try!(self.expect_identifier(Some("FROM")));
        self.consume_token();
        if let Some(&lexer::LexItem::Str(ref s)) = self.current_token() {
            log_file_name = s.clone();
        } else {
            return Err(format!("Expected String, got {:?}", self.current_token()));
        }

        self.consume_token();

        Ok(ASTNode::new(GrammarItem::LogFile { filename: log_file_name.into(), fields: log_file_fields }, None, None))
    }

    fn parse_condition(&mut self) -> Result<ASTNode, String> {
        try!(self.expect_identifier(Some("WHERE")));
        self.consume_token();
        let log_file_field = try!(self.expect_identifier(None));
        self.consume_token();

        let where_comparator;

        if let Ok(_) = self.expect_equals() {
            where_comparator = WhereComparator::StrictEquals;
            self.consume_token();
        } else if let Ok(_) = self.expect_identifier(Some("LIKE")) {
            where_comparator = WhereComparator::Like;
            self.consume_token();
        } else {
            return Err(format!("Expected '=' or LIKE, got {:?}", self.current_token()))
        }

        let log_where_clause_value = try!(self.parse_log_file_where_value());
        self.consume_token();

        Ok(ASTNode::new(GrammarItem::Condition { field: log_file_field, mode: where_comparator, value: log_where_clause_value }, None, None))
    }

    fn parse_limit(&mut self) -> Result<ASTNode, String> {
        try!(self.expect_identifier(Some("LIMIT")));
        self.consume_token();
        let direction;
        if let Ok(_) = self.expect_identifier(Some("LAST")) {
            direction = LimitDirection::Last;
            self.consume_token();
        } else {
            direction = LimitDirection::First;
        }

        let number_of_rows = try!(self.expect_number(None));

        Ok(ASTNode::new(GrammarItem::Limit { number_of_rows: number_of_rows, direction: direction }, None, None))
    }

    fn expect_select_field_list(&mut self) -> Result<Vec<String>, String> {
        let mut select_fields = vec!();

        while self.token_index < self.token_stream.len() {
            match self.current_token() {
                Some(&LexItem::Identifier(ref identifier)) => {
                    if identifier == "FROM" {
                        return Err("Expected Select Identifier, got keyword FROM".into());
                    }
                    select_fields.push(identifier.clone());
                    if let Some(&LexItem::Identifier(ref possible_from)) = self.next_token() {
                        if possible_from == "FROM" {
                            break;
                        }
                    }
                },
                Some(&LexItem::Comma) => {
                    if let Some(&LexItem::Identifier(ref possible_from)) = self.next_token() {
                        if possible_from == "FROM" {
                            return Err("Expected Identifier, got keyword FROM".into());
                        }
                    } else {
                        return Err(format!("Expected Identifier, got {:?}", self.next_token()));
                    }
                },
                _ => return Err(format!("Expected Identifier, got {:?}", self.current_token()))
            }

            self.consume_token();
        }

        Ok(select_fields)
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        self.token_stream = try!(lexer::tokenize(&self.query));
        self.token_index = 0;

        try!(self.expect_identifier(Some("SELECT")));
        self.consume_token();

        let log_file_node = try!(self.parse_log_file());
        let condition;
        let limit;

        if let Ok(_) = self.expect_identifier(Some("WHERE".into())) {
            condition = Some(Box::new(try!(self.parse_condition())));
        } else {
            condition = None;
        }

        if let Ok(_) = self.expect_identifier(Some("LIMIT".into())) {
            limit = Some(Box::new(try!(self.parse_limit())));
            self.consume_token();
        } else {
            limit = None;
        }

        try!(self.expect_eof());

        let log_result_node;
        if condition.is_some() || limit.is_some() {
            log_result_node = Some(Box::new(ASTNode::new(GrammarItem::LogResult, condition, limit)));
        } else {
            log_result_node = None;
        }

        Ok(ASTNode::new(GrammarItem::Query, Some(Box::new(log_file_node)), log_result_node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_delivers_tree_for_simple_query() {
        let query = "SELECT title FROM 'app.log' WHERE severity = 'error'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.entry, GrammarItem::Query);
        assert_eq!(ast.left.unwrap().entry, GrammarItem::LogFile { filename: "app.log".into(), fields: vec!("title".into()) });
        let right_node = ast.right.unwrap();
        assert_eq!(right_node.left.unwrap().entry, GrammarItem::Condition { field: "severity".into(), mode: WhereComparator::StrictEquals, value: "error".into() });
    }

    #[test]
    fn it_delivers_tree_for_query_with_multiple_select_fields() {
        let query = "SELECT title, severity, date FROM 'app.log' WHERE severity = 'error'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.entry, GrammarItem::Query);
        assert_eq!(ast.left.unwrap().entry, GrammarItem::LogFile { filename: "app.log".into(), fields: vec!("title".into(), "severity".into(), "date".into()) });
        let right_node = ast.right.unwrap();
        assert_eq!(right_node.left.unwrap().entry, GrammarItem::Condition { field: "severity".into(), mode: WhereComparator::StrictEquals, value: "error".into() });
    }

    #[test]
    fn it_fails_when_select_field_is_missing() {
        let query = "SELECT  FROM 'app.log' WHERE severity = 'error'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse();
        assert!(ast.is_err());
    }

    #[test]
    fn it_fails_when_from_keyword_is_missing() {
        let query = "SELECT title, severity, date 'app.log' WHERE severity = 'error'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse();
        assert!(ast.is_err());
    }

    #[test]
    fn it_fails_when_filename_is_missing() {
        let query = "SELECT title, severity, date FROM WHERE severity = 'error'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse();
        assert!(ast.is_err());
    }

    #[test]
    fn it_fails_when_where_keyword_is_missing() {
        let query = "SELECT title, severity, date FROM 'app.log' severity = 'error'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse();
        assert!(ast.is_err());
    }

    #[test]
    fn it_does_not_fail_when_where_clause_is_missing() {
        let query = "SELECT title, severity, date FROM 'app.log'".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse();
        let left_ast = ast.clone();
        let right_ast = ast.clone();

        assert!(left_ast.unwrap().left.is_some());
        assert!(right_ast.unwrap().right.is_none());
    }

    #[test]
    fn it_produces_ast_for_select_with_limit_10() {
        let query = "SELECT title, severity, date FROM 'app.log' LIMIT 10".into();
        let mut parser = Parser::new(query);

        let ast = parser.parse();
        assert!(ast.is_ok());
        let right_node = ast.unwrap().right.unwrap();
        assert_eq!(right_node.right.unwrap().entry, GrammarItem::Limit { number_of_rows: 10, direction: LimitDirection::First });
    }

    #[test]
    fn it_fails_when_limit_does_not_have_a_number() {
        let query = "SELECT title, severity, date FROM 'app.log' LIMIT".into();
        let mut parser = Parser::new(query);

        let ast = parser.parse();
        assert!(ast.is_err());
    }

    #[test]
    fn it_produces_ast_for_select_with_limit_last_10() {
        let query = "SELECT title, severity, date FROM 'app.log' LIMIT LAST 10".into();
        let mut parser = Parser::new(query);
        let ast = parser.parse();
        assert!(ast.is_ok());
        let right_node = ast.unwrap().right.unwrap();
        assert_eq!(right_node.right.unwrap().entry, GrammarItem::Limit { number_of_rows: 10, direction: LimitDirection::Last });
    }

    #[test]
    fn it_produces_ast_for_select_with_where_clause_and_limit() {
        let query = "SELECT title, severity FROM 'app.log' WHERE title = 'Network connection failed' LIMIT LAST 10".into();
        let mut parser = Parser::new(query);
        let ast_or_err = parser.parse();

        assert!(ast_or_err.is_ok());
        let ast = ast_or_err.unwrap();

        let right_node = *ast.right.unwrap().clone();


        let left_result_node = &right_node.left.unwrap();
        let right_result_node = &right_node.right.unwrap();

        assert_eq!(left_result_node.entry, GrammarItem::Condition { field: "title".into(), mode: WhereComparator::StrictEquals, value: "Network connection failed".into() });
        assert_eq!(right_result_node.entry, GrammarItem::Limit { number_of_rows: 10, direction: LimitDirection::Last });
    }

    #[test]
    fn it_returns_error_for_query_with_limit_and_where_in_the_wrong_order() {
        let query = "SELECT title, severity FROM 'app.log' LIMIT LAST 10 WHERE title = 'Network connection failed'".into();
        let mut parser = Parser::new(query);
        let expected_err = parser.parse();
        assert!(expected_err.is_err());
    }

    #[test]
    fn it_raises_an_error_when_limit_number_is_negative() {
        let query = "SELECT title, severity FROM 'app.log' LIMIT LAST -10".into();
        let mut parser = Parser::new(query);
        let expected_err = parser.parse();
        assert!(expected_err.is_err());
    }

    #[test]
    fn it_returns_ast_for_where_clause_with_like_operator() {
        let query = "SELECT title, severity FROM 'app.log' WHERE title LIKE 'dies, das'".into();

        let mut parser = Parser::new(query);
        let ast = parser.parse().unwrap();
        let right_node = *ast.right.unwrap().clone();
        let conditional_node = &right_node.left.unwrap();

        assert_eq!(conditional_node.entry, GrammarItem::Condition { field: "title".into(), mode: WhereComparator::Like, value: "dies, das".into() });
    }

    #[test]
    fn it_fails_when_where_does_not_have_like_or_equals() {
        let query = "SELECT title, severity FROM 'app.log' WHERE title 'dies, das'".into();

        let mut parser = Parser::new(query);
        let ast = parser.parse();

        assert!(ast.is_err());
    }

    #[test]
    fn it_fails_when_where_does_have_unexpected_keyword() {
        let query = "SELECT title, severity FROM 'app.log' WHERE title Foo 'dies, das'".into();

        let mut parser = Parser::new(query);
        let ast = parser.parse();

        assert!(ast.is_err());
    }

    #[test]
    fn it_fails_when_where_does_have_like_in_lower_case() {
        let query = "SELECT title, severity FROM 'app.log' WHERE title like 'dies, das'".into();

        let mut parser = Parser::new(query);
        let ast = parser.parse();

        assert!(ast.is_err());
    }
}