use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(String),
    StringLiteral(String),

    Equals,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    EqualsEquals,
    NotEquals,
    And,
    Or,
    Not,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    Semicolon,
    Dot,
}

pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

const KEYWORDS: &[(&str, &str)] = &[
    ("grrr", "Glrm"),
    ("mrrgglif", "Mrglif"),
    ("grrrfnrrg", "Mrglfn"),
    ("grrrblbl", "Mrglcall"),
    ("blgrrimport", "Mrglimport"),
    ("mrrg", "Mrrg"),
    ("mrgl", "Mrgl"),
    ("grl", "Grl"),
    ("grrrwhile", "Mrglwhile"),
    ("grrrswitch", "Mrglswitch"),
    ("blgrrstop", "Mrglbreak"),
    ("blgrrkeep", "Mrglcontinue"),
    ("grrprint", "Mrglprint"),
    ("grrread", "Mrglread"),
    ("grrrmath", "Mrglmath"),
    ("grrmap", "Mrglmap"),
    ("grrfilter", "Mrglfilter"),
    ("grrreduce", "Mrglreduce"),
    ("mrglspawn", "Mrglspawn"),
    ("grrsync", "Mrglsync"),
    ("grrarray", "Mrglarray"),
    ("mrrgstruct", "Mrglstruct"),
    ("mrglwait", "Mrglwait"),
    ("mrglasync", "Mrglasync"),
    ("mrglawait", "Mrglawait"),
    ("fshpool", "Mrglpool"),
    ("fshpoolsize", "MrglpoolSize"),
    ("mrglfish", "Mrglfish"),
    ("mrglbubble", "Mrglbubble"),
    ("mrgltide", "Mrgltide"),
    ("mrglshell", "Mrglshell"),
    ("mrglwave", "Mrglwave"),
    ("mrglwhen", "Mrglwhen"),
    ("mrglcatch", "Mrglcatch"),
    ("mrglswim", "Mrglswim"),
    ("mrglschool", "Mrglschool"),
    ("mrglsplash", "Mrglsplash"),
    ("mrglcurrent", "Mrglcurrent"),
    ("mrglpearl", "Mrglpearl"),
    ("mrglcoral", "Mrglcoral"),
    ("mrglif", "Mrglif"),
    ("mrglelse", "Mrglelse"),
    ("grrrtn", "Mrglreturn")
];

pub fn tokenize(source: &str) -> Vec<SpannedToken> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut column = 1;

    macro_rules! push_token {
        ($tok:expr) => {
            tokens.push(SpannedToken {
                token: $tok,
                line,
                column,
            });
        };
    }

    while let Some(c) = chars.next() {
        match c {
            ' ' | '\t' => column += 1,
            '\n' => {
                line += 1;
                column = 1;
            }
            '/' if chars.peek() == Some(&'/') => {
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        line += 1;
                        column = 1;
                        break;
                    } else {
                        column += 1;
                    }
                }
            }
            '+' => push_token!(Token::Plus),
            '-' => push_token!(Token::Minus),
            '*' => push_token!(Token::Multiply),
            '%' => push_token!(Token::Modulo),
            '=' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    push_token!(Token::EqualsEquals);
                } else {
                    push_token!(Token::Equals);
                }
            }
            '<' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    push_token!(Token::LessThanOrEqual);
                } else {
                    push_token!(Token::LessThan);
                }
            }
            '>' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    push_token!(Token::GreaterThanOrEqual);
                } else {
                    push_token!(Token::GreaterThan);
                }
            }
            '!' => {
                if chars.peek() == Some(&'=') {
                    chars.next();
                    push_token!(Token::NotEquals);
                } else {
                    push_token!(Token::Not);
                }
            }
            '&' => {
                if chars.peek() == Some(&'&') {
                    chars.next();
                    push_token!(Token::And);
                }
            }
            '|' => {
                if chars.peek() == Some(&'|') {
                    chars.next();
                    push_token!(Token::Or);
                }
            }
            '(' => push_token!(Token::LeftParen),
            ')' => push_token!(Token::RightParen),
            '{' => push_token!(Token::LeftBrace),
            '}' => push_token!(Token::RightBrace),
            '[' => push_token!(Token::LeftBracket),
            ']' => push_token!(Token::RightBracket),
            ',' => push_token!(Token::Comma),
            ':' => push_token!(Token::Colon),
            ';' => push_token!(Token::Semicolon),
            '.' => push_token!(Token::Dot),
            '"' => {
                let mut string = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    }
                    string.push(ch);
                }
                push_token!(Token::StringLiteral(string));
            }
            ch if ch.is_alphabetic() => {
                let mut ident = ch.to_string();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Some(&(_, name)) = KEYWORDS.iter().find(|&&(k, _)| k == ident) {
                    push_token!(Token::Keyword(name.to_string()));
                } else {
                    push_token!(Token::Identifier(ident));
                }
            }
            ch if ch.is_digit(10) => {
                let mut num = ch.to_string();
                while let Some(&ch) = chars.peek() {
                    if ch.is_digit(10) {
                        num.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                push_token!(Token::Number(num));
            }
            _ => panic!("Caractere inválido: {}", c),
        }
    }

    tokens
}