mod lexer;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum GrammarItem {
    Query,
    LogFile { field: String, filename: String },
    Condition { field: String, value: String }
}

#[derive(Debug)]
pub struct ASTNode {
    left: Option<Box<ASTNode>>,
    right: Option<Box<ASTNode>>,
    entry: GrammarItem,
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

fn expect_equals(actual: &lexer::LexItem) -> Result<(), String> {
    match *actual {
        lexer::LexItem::Equals(_) => {
            Ok(())
        },
        _ => {
            Err(format!("Expected assign, got {:?}", actual))
        }
    }
}

fn expect_identifier(actual: &lexer::LexItem, identifier_value: Option<&str>) -> Result<(), String> {
    match actual {
        &lexer::LexItem::Identifier(ref identifier) => {
            if identifier_value.is_some() {
                if identifier_value.unwrap() == identifier {
                    Ok(())
                } else {
                    Err(format!("Expected Identifier {:?}, got {:?}", identifier_value.unwrap(), actual))
                }
            } else {
                Ok(())
            }
        },
        _ => { Err(format!("Expected Identifier, got {:?}", actual)) }
    }
}

fn parse_log_file_field(tokens: &Vec<lexer::LexItem>, index: usize) -> Result<String, String> {
    if let &lexer::LexItem::Identifier(ref s) = &tokens[index] {
        Ok(s.clone())
    } else {
        Err(format!("Expected Identifier, got {:?}", tokens[index]))
    }
}

fn parse_log_file_where_value(tokens: &Vec<lexer::LexItem>) -> Result<String, String> {
    if let &lexer::LexItem::Str(ref s) = &tokens[7] {
        Ok(s.clone())
    } else {
        Err(format!("Expected String, got {:?}", tokens[7]))
    }
}

fn parse_log_file(tokens: &Vec<lexer::LexItem>) -> Result<ASTNode, String> {
    try!(expect_identifier(&tokens[0], Some("SELECT")));

    let log_file_name;
    let log_file_field = try!(parse_log_file_field(&tokens, 1));

    try!(expect_identifier(&tokens[2], Some("FROM")));
    if let &lexer::LexItem::Str(ref s) = &tokens[3] {
        log_file_name = s.clone();
    } else {
        return Err(format!("Expected String, got {:?}", tokens[3]));
    }

    Ok(ASTNode::new(GrammarItem::LogFile { filename: log_file_name.into(), field: log_file_field }, None, None))
}

fn parse_condition(tokens: &Vec<lexer::LexItem>) -> Result<ASTNode, String> {
    try!(expect_identifier(&tokens[4], Some("WHERE")));
    let log_file_field = try!(parse_log_file_field(&tokens, 5));
    try!(expect_equals(&tokens[6]));
    let log_where_clause_value = try!(parse_log_file_where_value(&tokens));

    Ok(ASTNode::new(GrammarItem::Condition { field: log_file_field, value: log_where_clause_value }, None, None))
}

fn parse_query(tokens: &Vec<lexer::LexItem>) -> Result<ASTNode, String> {
    let log_file_node = try!(parse_log_file(&tokens));
    let condition_node = try!(parse_condition(&tokens));
    Ok(ASTNode::new(GrammarItem::Query, Some(Box::new(log_file_node)), Some(Box::new(condition_node))))
}

pub fn get_ast_for_query(query: String) -> Result<ASTNode, String> {
    let tokens = try!(lexer::tokenize(&query));

    parse_query(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_delivers_tree_for_simple_query() {
        let query = "SELECT title FROM 'app.log' WHERE severity = 'error'".into();
        let ast = get_ast_for_query(query).unwrap();
        assert_eq!(ast.entry, GrammarItem::Query);
        assert_eq!(ast.left.unwrap().entry, GrammarItem::LogFile { filename: "app.log".into(), field: "title".into() });
        assert_eq!(ast.right.unwrap().entry, GrammarItem::Condition { field: "severity".into(), value: "error".into() });
    }
}