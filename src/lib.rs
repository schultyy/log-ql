mod lexer;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum GrammarItem {
    Query,
    LogFile { field: String, filename: String },
    Condition
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

fn parse_query(tokens: &Vec<lexer::LexItem>, pos: usize) -> Result<(ASTNode, usize), String> {
    try!(expect_identifier(&tokens[0], Some("SELECT")));

    let log_file_field;
    let log_file_name;

    if let &lexer::LexItem::Identifier(ref s) = &tokens[1] {
        log_file_field = s.clone();
    } else {
        return Err(format!("Expected Identifier, got {:?}", tokens[1]));
    }

    try!(expect_identifier(&tokens[2], Some("FROM")));
    if let &lexer::LexItem::Str(ref s) = &tokens[3] {
        log_file_name = s.clone();
    } else {
        return Err(format!("Expected String, got {:?}", tokens[3]));
    }

    let log_file_node = Some(Box::new(ASTNode::new(GrammarItem::LogFile { filename: log_file_name.into(), field: log_file_field }, None, None)));
    Ok((ASTNode::new(GrammarItem::Query, log_file_node, None), tokens.len()))
}

pub fn get_ast_for_query(query: String) -> Result<ASTNode, String> {
    let tokens = try!(lexer::tokenize(&query));

    parse_query(&tokens, 0).and_then(|(n, i)| if i == tokens.len() {
        Ok(n)
    } else {
        Err(format!("Expected end of input, found {:?} at {}", tokens[i], i))
    })
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
    }
}