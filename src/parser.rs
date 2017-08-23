use lexer;
use lexer::LexItem;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum GrammarItem {
    Query,
    LogFile { field: String, filename: String },
    Condition { field: String, value: String }
}

#[derive(Debug)]
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

    fn current_token(&self) -> &lexer::LexItem {
        &self.token_stream[self.token_index]
    }

    fn next_token(&mut self) {
        self.token_index += 1;
    }

    fn expect_equals(&self) -> Result<(), String> {
        match *self.current_token() {
            lexer::LexItem::Equals(_) => {
                Ok(())
            },
            _ => {
                Err(format!("Expected assign, got {:?}", self.current_token()))
            }
        }
    }

    fn expect_identifier(&self, identifier_value: Option<&str>) -> Result<String, String> {
        match self.current_token() {
            &lexer::LexItem::Identifier(ref identifier) => {
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

    fn parse_log_file_where_value(&self) -> Result<String, String> {
        if let &lexer::LexItem::Str(ref s) = self.current_token() {
            Ok(s.clone())
        } else {
            Err(format!("Expected String, got {:?}", self.current_token()))
        }
    }

    fn parse_log_file(&mut self) -> Result<ASTNode, String> {
        let log_file_name;
        let log_file_field = try!(self.expect_select_field_list());
        self.next_token();

        try!(self.expect_identifier(Some("FROM")));
        self.next_token();
        if let &lexer::LexItem::Str(ref s) = self.current_token() {
            log_file_name = s.clone();
        } else {
            return Err(format!("Expected String, got {:?}", self.current_token()));
        }

        self.next_token();

        Ok(ASTNode::new(GrammarItem::LogFile { filename: log_file_name.into(), field: log_file_field[0].clone() }, None, None))
    }

    fn parse_condition(&mut self) -> Result<ASTNode, String> {
        try!(self.expect_identifier(Some("WHERE")));
        self.next_token();
        let log_file_field = try!(self.expect_identifier(None));
        self.next_token();
        try!(self.expect_equals());
        self.next_token();
        let log_where_clause_value = try!(self.parse_log_file_where_value());
        self.next_token();

        Ok(ASTNode::new(GrammarItem::Condition { field: log_file_field, value: log_where_clause_value }, None, None))
    }

    fn expect_select_field_list(&mut self) -> Result<Vec<String>, String> {
        let mut select_fields = vec!();

        while self.token_index < self.token_stream.len() {
            match self.current_token() {
                &LexItem::Identifier(ref identifier) => {
                    if identifier == "FROM" {
                        return Err("Expected Select Identifier, got keyword FROM".into());
                    }
                    select_fields.push(identifier.clone());
                    break;
                },
                _ => return Err(format!("Expected Identifier, got {:?}", self.current_token()))
            }

            self.token_index += 1;
        }

        Ok(select_fields)
    }

    pub fn parse(&mut self) -> Result<ASTNode, String> {
        self.token_stream = try!(lexer::tokenize(&self.query));
        self.token_index = 0;

        try!(self.expect_identifier(Some("SELECT")));
        self.next_token();

        let log_file_node = try!(self.parse_log_file());
        let condition_node = try!(self.parse_condition());

        Ok(ASTNode::new(GrammarItem::Query, Some(Box::new(log_file_node)), Some(Box::new(condition_node))))
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
        assert_eq!(ast.left.unwrap().entry, GrammarItem::LogFile { filename: "app.log".into(), field: "title".into() });
        assert_eq!(ast.right.unwrap().entry, GrammarItem::Condition { field: "severity".into(), value: "error".into() });
    }
}