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

#[derive(Clone, Debug)]
enum Literal<'a> {
    String(&'a str),
    Number(f64),
    Bool(bool),
    Nil,
}

impl<'a> std::fmt::Display for Literal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "Nil"),
            s => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Debug)]
struct Token<'a> {
    type_: TokenType,
    lexeme: &'a [char],
    literal: Option<Literal<'a>>,
    line: usize,
}

impl<'a> Token<'a> {
    pub fn new(
        type_: TokenType,
        lexeme: &'a [char],
        literal: Option<Literal<'a>>,
        line: usize,
    ) -> Self {
        Self {
            type_,
            lexeme,
            literal,
            line,
        }
    }
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(lit) = &self.literal {
            write!(
                f,
                "{:?} {} {}",
                self.type_,
                &String::from_iter(self.lexeme),
                lit,
            )
        } else {
            write!(
                f,
                "{:?} {} {}",
                self.type_,
                &String::from_iter(self.lexeme),
                "None",
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
        self.buffer.push(Token::new(
            type_,
            &self.source[self.l..self.r],
            literal,
            self.line,
        ));
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

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        self.source[self.r]
    }

    // the rules that determine how a particular language groups characters into
    // lexemes are called its lexical grammar.
    pub fn scan_tokens(&mut self) -> impl Iterator<Item = Token<'a>> {
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
                '/' => {
                    if self.try_match('/') {
                        // A comment goes until the end of the line.
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.r += 1;
                        }
                    } else {
                        self.add_token(TokenType::Slash, None);
                    }
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

        self.buffer
            .push(Token::new(TokenType::Eof, &[], None, self.line));
        self.buffer.clone().into_iter()
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
    for token in scanner.scan_tokens() {
        println!("{}", token);
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
