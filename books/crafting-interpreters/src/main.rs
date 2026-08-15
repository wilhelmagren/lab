use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::process;

static mut HAD_ERROR: bool = false;

// Lexemes and Tokens
//
// Lexical analysis, scan through list of characters and group them together
// into the smallest sequences that sill represent something. Each group/blob
// is called a lexeme.

#[derive(Clone, Debug)]
enum TokenType {
    // single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // one or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl TokenType {
    pub fn from_identifier(chars: &[char]) -> Option<Self> {
        let ident = chars.iter().cloned().collect::<String>();
        let tt = match ident.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => {
                return None;
            }
        };
        Some(tt)
    }
}

#[derive(Clone, Debug)]
enum Literal<'a> {
    String(&'a [char]),
    Number(f64),
}

impl<'a> std::fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => {
                write!(f, "{}", s.iter().copied().collect::<String>())
            }
            Self::Number(n) => write!(f, "{}", n),
        }
    }
}

#[derive(Clone, Debug)]
struct Token<'a> {
    type_: TokenType,
    lexeme: &'a [char],
    literal: Option<Literal<'a>>,
    indices: (usize, usize),
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(
        type_: TokenType,
        lexeme: &'a [char],
        literal: Option<Literal<'a>>,
        indices: (usize, usize),
        line: usize,
    ) -> Self {
        Self {
            type_,
            lexeme,
            literal,
            indices,
            line,
        }
    }
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(lit) = &self.literal {
            write!(
                f,
                "Token({:?}, lexeme='{}', literal='{}', line={}, indices=[{},{}))",
                self.type_,
                &String::from_iter(self.lexeme),
                lit,
                self.line,
                self.indices.0,
                self.indices.1,
            )
        } else {
            write!(
                f,
                "Token({:?}, lexeme='{}', literal=None, line={}, indices=[{},{}))",
                self.type_,
                &String::from_iter(self.lexeme),
                self.line,
                self.indices.0,
                self.indices.1,
            )
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Scanner<'a> {
    source: &'a [char],

    l: usize,
    r: usize,

    line: usize,
    buffer: Vec<Token<'a>>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a [char]) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }

    fn add_token(&mut self, type_: TokenType, literal: Option<Literal<'a>>) {
        let t = Token::new(
            type_,
            &self.source[self.l..self.r],
            literal,
            (self.l, self.r),
            self.line,
        );
        self.buffer.push(t);
    }

    fn try_match(&mut self, expected: char) -> bool {
        if self.source[self.r] != expected {
            return false;
        }
        self.r += 1;
        return true;
    }

    fn is_at_end(&self) -> bool {
        return self.r >= self.source.len();
    }

    fn is_digit(&self, c: char) -> bool {
        // this works cus chars are ASCII numbers ordered
        c >= '0' && c <= '9'
    }

    fn is_alpha(&self, c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
    }

    fn is_alpha_numeric(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.r]
    }

    fn peek_next(&self) -> char {
        let next = self.r + 1;
        if next >= self.source.len() {
            return '\0';
        }
        self.source[next]
    }

    // the rules that determine how a particular language groups characters into
    // lexemes are called its lexical grammar.
    pub fn scan_tokens(&mut self) -> Option<impl Iterator<Item = Token<'a>>> {
        while !self.is_at_end() {
            let c = self.source[self.r];

            self.l = self.r;
            self.r += 1;

            match c {
                '(' => self.add_token(TokenType::LeftParen, None),
                ')' => self.add_token(TokenType::RightParen, None),
                '{' => self.add_token(TokenType::LeftBrace, None),
                '}' => self.add_token(TokenType::RightBrace, None),
                ',' => self.add_token(TokenType::Comma, None),
                '.' => self.add_token(TokenType::Dot, None),
                '-' => self.add_token(TokenType::Minus, None),
                '+' => self.add_token(TokenType::Plus, None),
                ';' => self.add_token(TokenType::Semicolon, None),
                '*' => self.add_token(TokenType::Star, None),
                '!' => {
                    let tt = match self.try_match('=') {
                        true => TokenType::BangEqual,
                        false => TokenType::Bang,
                    };
                    self.add_token(tt, None);
                }
                '=' => {
                    let tt = match self.try_match('=') {
                        true => TokenType::EqualEqual,
                        false => TokenType::Equal,
                    };
                    self.add_token(tt, None);
                }
                '<' => {
                    let tt = match self.try_match('=') {
                        true => TokenType::LessEqual,
                        false => TokenType::Less,
                    };
                    self.add_token(tt, None);
                }
                '>' => {
                    let tt = match self.try_match('=') {
                        true => TokenType::GreaterEqual,
                        false => TokenType::Greater,
                    };
                    self.add_token(tt, None);
                }
                // Challenges 4: Add support for C-style `/* ... */` block comments.
                '/' => {
                    //where is self.r?
                    let np = self.peek();

                    // A normal comment goes until the end of the line.
                    if np == '/' {
                        self.r += 1;
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.r += 1;
                        }
                    } else if np == '*' {
                        self.r += 1;

                        // a block comment goes until the next time we see */
                        let mut peeked = self.peek();
                        while !self.is_at_end() {
                            if peeked == '*' && self.peek_next() == '/' {
                                // skip those chars
                                self.r += 2;
                                break;
                            }

                            // need to do this before peeking again
                            // otherwise we miss counting a \n
                            if peeked == '\n' {
                                self.line += 1;
                            }

                            self.r += 1;
                            peeked = self.peek();
                        }
                    } else {
                        self.add_token(TokenType::Slash, None);
                    }
                }
                // noop
                ' ' | '\r' | '\t' => {}
                '\n' => {
                    self.line += 1;
                }
                '"' => {
                    let mut peeked = self.peek();
                    while peeked != '"' && !self.is_at_end() {
                        if peeked == '\n' {
                            self.line += 1;
                        }
                        self.r += 1;
                        peeked = self.peek()
                    }

                    if self.is_at_end() {
                        error(self.line, "Unterminated string.");
                        return None;
                    }

                    self.r += 1;

                    self.add_token(
                        TokenType::String,
                        Some(Literal::String(&self.source[self.l + 1..self.r - 1])),
                    );
                }
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                    while self.is_digit(self.peek()) {
                        self.r += 1;
                    }

                    // looking past the decimal point requires a second character of
                    // lookahead since we don't want to soncume the '.' until we're
                    // really sure there is a digit after it
                    if self.peek() == '.' && self.is_digit(self.peek_next()) {
                        self.r += 1;
                        while self.is_digit(self.peek()) {
                            self.r += 1;
                        }
                    }
                    self.add_token(
                        TokenType::Number,
                        // this is really ugly but yea, that's we way we have to do it
                        Some(Literal::Number(
                            self.source[self.l..self.r]
                                .iter()
                                .copied()
                                .collect::<String>()
                                .parse::<f64>()
                                .unwrap(),
                        )),
                    );
                }
                'A'..='Z' | 'a'..='z' | '_' => {
                    while self.is_alpha_numeric(self.peek()) {
                        self.r += 1;
                    }

                    let tt = match TokenType::from_identifier(&self.source[self.l..self.r]) {
                        Some(tt) => tt,
                        None => TokenType::Identifier,
                    };

                    self.add_token(tt, None);
                }
                _ => {
                    error(
                        self.line,
                        &format!(
                            "Unexpected character '{}' (column={}).",
                            c.escape_default(),
                            self.r
                        ),
                    );
                }
            }
        }

        self.buffer.push(Token::new(
            TokenType::Eof,
            &[],
            None,
            (self.l, self.r),
            self.line,
        ));
        Some(self.buffer.clone().into_iter())
    }
}

fn report(line: usize, where_: &str, msg: &str) {
    eprintln!("[line {}] Error{}: {}", line, where_, msg);
}

fn error(line: usize, msg: &str) {
    report(line, "", msg);
    unsafe { HAD_ERROR = true };
}

fn run(source: String) {
    let charvec = source.chars().collect::<Vec<char>>();
    let mut scanner = Scanner::new(&charvec);
    if let Some(iterator) = scanner.scan_tokens() {
        for token in iterator {
            println!("{}", token);
        }
    }
}

fn run_file(file_path: &str) {
    run(String::from_utf8(fs::read(file_path).unwrap()).unwrap());
}

// interactive prompt: REPL, from Lisp, based af
fn run_prompt() {
    let mut buf = String::with_capacity(1000);

    loop {
        buf.clear();

        print!("> ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut buf).unwrap();

        if buf.is_empty() {
            break;
        }

        run(buf.clone());
    }
}

fn main() -> process::ExitCode {
    let mut args = env::args();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
        return process::ExitCode::FAILURE;
    } else if args.len() == 2 {
        run_file(&args.nth(1).unwrap());
    } else {
        run_prompt();
    }

    process::ExitCode::SUCCESS
}
