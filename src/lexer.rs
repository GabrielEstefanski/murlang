use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Number(String),
    StringLiteral(String),

    Assign,       // =
    Plus,         // +
    Minus,        // -
    Multiply,     // *
    Divide,       // /
    Modulo,       // %
    LessThan,     // <
    GreaterThan,  // >
    LessEqual,    // <=
    GreaterEqual, // >=
    Equal,        // ==
    NotEqual,     // !=
    And,          // &&
    Or,           // ||
    Not,          // !

    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Colon,        // :
    Semicolon,    // ;
    Dot,          // .
}

pub struct SpannedToken {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

const KEYWORDS: &[(&str, &str)] = &[
    ("grrr", "var"),
    ("grlbrr", "if"),
    ("grrrfnrrg", "fn"),
    ("grrrblbl", "call"),
    ("blgrrimport", "import"),
    ("mrrg", "for"),
    ("mrgl", "begin"),
    ("grl", "end"),
    ("gglrbl ", "while"),
    ("murrrgh", "switch"),
    ("blgrrstop", "break"),
    ("blgrrkeep", "continue"),
    ("glglrr", "print"),
    ("glurp", "read"),
    ("mthgrrr", "math"),
    ("splurg", "spawn"),
    ("grrsync", "sync"),
    ("grrip", "array"),
    ("rrkgr", "struct"),
    ("mrglwait", "wait"),
    ("mrglasync", "async"),
    ("mrglawait", "await"),
    ("fshpool", "threadpool"),
    ("fshpoolsize", "poolsize"),
    ("mrglwhen", "when"),
    ("mrglcatch", "catch"),
    ("mrglswim", "try"),
    ("mrglschool", "group"),
    ("blrrgl", "else"),
    ("grrrtn", "return"),
    ("blbtxt", "text"),
    ("numblrr", "number")
];

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Lexer {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(c) = self.chars.next() {
            match c {
                ' ' | '\t' => self.column += 1,
                '\r' => {},
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                }
                 
                '+' | '-' | '*' | '/' | '%' | '=' | '<' | '>' | '!' | '&' | '|' => {
                    if let Ok(token) = self.process_operator(c) {
                        if !(c == '/' && token.line == 0 && token.column == 0) {
                            tokens.push(token);
                        }
                    } else if let Err(e) = self.process_operator(c) {
                        return Err(e);
                    }
                }
                
                '(' | ')' | '{' | '}' | '[' | ']' | ',' | ':' | ';' | '.' => {
                    tokens.push(self.process_delimiter(c));
                }
                
                '"' => {
                    match self.process_string() {
                        Ok(token) => tokens.push(token),
                        Err(e) => return Err(e),
                    }
                }
                
                ch if ch.is_alphabetic() => {
                    tokens.push(self.process_identifier(ch));
                }
                
                ch if ch.is_digit(10) => {
                    tokens.push(self.process_number(ch));
                }
                
                _ => {
                    return Err(LexerError {
                        message: format!("Invalid character: '{}' at line {} column {}", c, self.line, self.column),
                        line: self.line,
                        column: self.column,
                    });
                }
            }
        }

        Ok(tokens)
    }

    fn process_identifier(&mut self, first_char: char) -> SpannedToken {
        let start_column = self.column;
        let mut ident = first_char.to_string();
        self.column += 1;
        
        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(self.chars.next().unwrap());
                self.column += 1;
            } else {
                break;
            }
        }
        
        if let Some(&(_, name)) = KEYWORDS.iter().find(|&&(k, _)| k == ident) {
            SpannedToken {
                token: Token::Keyword(name.to_string()),
                line: self.line,
                column: start_column,
            }
        } else {
            SpannedToken {
                token: Token::Identifier(ident),
                line: self.line,
                column: start_column,
            }
        }
    }

    fn process_number(&mut self, first_digit: char) -> SpannedToken {
        let start_column = self.column;
        let mut num = first_digit.to_string();
        self.column += 1;
        
        while let Some(&ch) = self.chars.peek() {
            if ch.is_digit(10) || ch == '.' && !num.contains('.') {
                num.push(self.chars.next().unwrap());
                self.column += 1;
            } else {
                break;
            }
        }
        
        SpannedToken {
            token: Token::Number(num),
            line: self.line,
            column: start_column,
        }
    }

    fn process_string(&mut self) -> Result<SpannedToken, LexerError> {
        let start_column = self.column;
        self.column += 1;
        let mut string = String::new();
        let mut escaped = false;

        while let Some(ch) = self.chars.next() {
            self.column += 1;
            
            if escaped {
                match ch {
                    'n' => string.push('\n'),
                    't' => string.push('\t'),
                    'r' => string.push('\r'),
                    '\\' => string.push('\\'),
                    '"' => string.push('"'),
                    _ => string.push(ch),
                }
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                break;
            } else {
                string.push(ch);
            }
        }
        
        if escaped {
            return Err(LexerError {
                message: format!("Unterminated escape sequence in string at line {} column {}", self.line, start_column),
                line: self.line,
                column: start_column,
            });
        }
        
        Ok(SpannedToken {
            token: Token::StringLiteral(string),
            line: self.line,
            column: start_column,
        })
    }

    fn process_operator(&mut self, operator: char) -> Result<SpannedToken, LexerError> {
        let start_column = self.column;
        self.column += 1;
        
        let token = match operator {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => {
                if self.chars.peek() == Some(&'/') {
                    self.chars.next();
                    self.column += 1;
                    
                    while let Some(ch) = self.chars.next() {
                        if ch == '\n' {
                            break;
                        } else {
                            self.column += 1;
                        }
                    }
                    
                    return Ok(SpannedToken {
                        token: Token::Divide,
                        line: 0,
                        column: 0,
                    });
                } else {
                    Token::Divide
                }
            },
            '%' => Token::Modulo,
            '=' => {
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    self.column += 1;
                    Token::Equal
                } else {
                    Token::Assign
                }
            },
            '<' => {
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    self.column += 1;
                    Token::LessEqual
                } else {
                    Token::LessThan
                }
            },
            '>' => {
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    self.column += 1;
                    Token::GreaterEqual
                } else {
                    Token::GreaterThan
                }
            },
            '!' => {
                if self.chars.peek() == Some(&'=') {
                    self.chars.next();
                    self.column += 1;
                    Token::NotEqual
                } else {
                    Token::Not
                }
            },
            '&' => {
                if self.chars.peek() == Some(&'&') {
                    self.chars.next();
                    self.column += 1;
                    Token::And
                } else {
                    return Err(LexerError {
                        message: format!("Invalid token: expected '&&', found single '&' at line {} column {}", self.line, start_column),
                        line: self.line,
                        column: start_column,
                    });
                }
            },
            '|' => {
                if self.chars.peek() == Some(&'|') {
                    self.chars.next();
                    self.column += 1;
                    Token::Or
                } else {
                    return Err(LexerError {
                        message: format!("Invalid token: expected '||', found single '|' at line {} column {}", self.line, start_column),
                        line: self.line,
                        column: start_column,
                    });
                }
            },
            _ => unreachable!("Undefined operator")
        };
        
        Ok(SpannedToken {
            token,
            line: self.line,
            column: start_column,
        })
    }

    fn process_delimiter(&mut self, delimiter: char) -> SpannedToken {
        let start_column = self.column;
        self.column += 1;
        
        let token = match delimiter {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            '[' => Token::LeftBracket,
            ']' => Token::RightBracket,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '.' => Token::Dot,
            _ => unreachable!("Undefined delimiter")
        };
        
        SpannedToken {
            token,
            line: self.line,
            column: start_column,
        }
    }
}

pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexerError> {
    let mut lexer = Lexer::new(source);
    lexer.tokenize()
}