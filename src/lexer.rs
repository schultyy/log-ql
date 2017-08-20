use std::iter::Peekable;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum LexItem {
  Identifier(String),
  Str(String),
  Equals(char),
}

fn consume_string<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> String {
  let mut resulting_str = String::from("");

  while let Some(ch) = iter.peek().map(|c| *c) {
    if ch == '\'' {
      break;
    } else {
      resulting_str.push(ch.clone());
      iter.next();
    }
  }

  resulting_str
}

fn consume_identifier<T: Iterator<Item = char>>(iter: &mut Peekable<T>) -> String {
  let mut resulting_str = String::from("");

  while let Some(ch) = iter.peek().map(|c| *c) {
    if ch.is_alphabetic() {
      resulting_str.push(ch.clone());
      iter.next();
    } else {
      break;
    }
  }

  resulting_str
}

pub fn tokenize(input: &String) -> Result<Vec<LexItem>, String> {
  let mut result = Vec::new();

  let mut it = input.chars().peekable();

  while let Some(&ch) = it.peek() {
    match ch {
      '\'' => {
        it.next();
        let string = consume_string(&mut it);
        result.push(LexItem::Str(string));
        it.next();
      },
      '=' => {
        result.push(LexItem::Equals('='));
        it.next();
      },
      ' ' => { it.next(); },
      _ => {
        if ch.is_alphabetic() {
          let string = consume_identifier(&mut it);
          result.push(LexItem::Identifier(string));
          it.next();
        } else {
          return Err(format!("Unexpected char {}", ch));
        }
      }
    }
  }

  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_string_consumes_until_quote() {
    let test_str = "'app.log'";
    let mut iter = test_str.chars().peekable();
    iter.next();
    let actual_str = consume_string(&mut iter);
    assert_eq!(actual_str, "app.log");
  }

  #[test]
  fn it_tokenizes_simple_select_where() {
    let results = tokenize(&"SELECT type FROM 'app.log' WHERE type = 'error'".into()).unwrap();
    assert_eq!(results[0], super::LexItem::Identifier("SELECT".into()));
    assert_eq!(results[1], super::LexItem::Identifier("type".into()));
    assert_eq!(results[2], super::LexItem::Identifier("FROM".into()));
    assert_eq!(results[3], super::LexItem::Str("app.log".into()));

    assert_eq!(results[4], super::LexItem::Identifier("WHERE".into()));
    assert_eq!(results[5], super::LexItem::Identifier("type".into()));
    assert_eq!(results[6], super::LexItem::Equals('='.into()));
    assert_eq!(results[7], super::LexItem::Str("error".into()));
  }
}