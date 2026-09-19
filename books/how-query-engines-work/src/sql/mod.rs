use crate::logical::plan::LogicalPlan;

#[derive(Clone, Debug)]
pub enum TokenType {
    // single-character tokens
    LeftParen,
    RightParen,
    Minus,
    Plus,
    Star,
    Slash,
    Semicolon,
    Comma,

    // comparisons
    Eq,
    Neq,
    Gt,
    GtEq,
    Lt,
    LtEq,

    // Keywords
    Select,
    From,
    Where,
    Limit,
    As,
    Left,
    Right,
    Inner,
    Join,
    Group,
    Order,
    By,
    Asc,
    Desc,
    Explain,
    Avg,
    Min,
    Max,
    Sum,

    // conditionals,
    And,
    Or,
    Xor,
    Is,
    Not,
    Null,
    In,
    Any,
    All,
    Exists,

    // referencing table or column
    Identifier,
}

impl From<&[char]> for TokenType {
    // this is lowercase
    fn from(value: &[char]) -> Self {
        match value
            .iter()
            .cloned()
            .collect::<String>()
            .to_lowercase()
            .as_str()
        {
            // single-character tokens
            "(" => TokenType::LeftParen,
            ")" => TokenType::RightParen,
            "-" => TokenType::Minus,
            "+" => TokenType::Plus,
            "*" => TokenType::Star,
            "/" => TokenType::Slash,
            ";" => TokenType::Semicolon,
            "," => TokenType::Comma,

            // comparisons
            "=" => TokenType::Eq,
            "!=" => TokenType::Neq,
            ">" => TokenType::Gt,
            ">=" => TokenType::GtEq,
            "<" => TokenType::Lt,
            "<=" => TokenType::LtEq,

            // keywords
            "select" => TokenType::Select,
            "from" => TokenType::From,
            "where" => TokenType::Where,
            "limit" => TokenType::Limit,
            "as" => TokenType::As,
            "left" => TokenType::Left,
            "right" => TokenType::Right,
            "inner" => TokenType::Inner,
            "join" => TokenType::Join,
            "group" => TokenType::Group,
            "order" => TokenType::Order,
            "by" => TokenType::By,
            "asc" => TokenType::Asc,
            "desc" => TokenType::Desc,
            "explain" => TokenType::Explain,
            "avg" => TokenType::Avg,
            "min" => TokenType::Min,
            "max" => TokenType::Max,
            "sum" => TokenType::Sum,

            // conditionals
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "xor" => TokenType::Xor,
            "is" => TokenType::Is,
            "not" => TokenType::Not,
            "null" => TokenType::Null,
            "in" => TokenType::In,
            "any" => TokenType::Any,
            "all" => TokenType::All,
            "exists" => TokenType::Exists,

            // must be table or col reference?
            _ => TokenType::Identifier,
        }
    }
}

impl From<&str> for TokenType {
    // this is lowercase
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            // single-character tokens
            "(" => TokenType::LeftParen,
            ")" => TokenType::RightParen,
            "-" => TokenType::Minus,
            "+" => TokenType::Plus,
            "*" => TokenType::Star,
            "/" => TokenType::Slash,
            ";" => TokenType::Semicolon,
            "," => TokenType::Comma,

            // comparisons
            "=" => TokenType::Eq,
            "!=" => TokenType::Neq,
            ">" => TokenType::Gt,
            ">=" => TokenType::GtEq,
            "<" => TokenType::Lt,
            "<=" => TokenType::LtEq,

            // keywords
            "select" => TokenType::Select,
            "from" => TokenType::From,
            "where" => TokenType::Where,
            "limit" => TokenType::Limit,
            "as" => TokenType::As,
            "left" => TokenType::Left,
            "right" => TokenType::Right,
            "inner" => TokenType::Inner,
            "join" => TokenType::Join,
            "group" => TokenType::Group,
            "order" => TokenType::Order,
            "by" => TokenType::By,
            "asc" => TokenType::Asc,
            "desc" => TokenType::Desc,
            "explain" => TokenType::Explain,
            "avg" => TokenType::Avg,
            "min" => TokenType::Min,
            "max" => TokenType::Max,
            "sum" => TokenType::Sum,

            // conditionals
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "xor" => TokenType::Xor,
            "is" => TokenType::Is,
            "not" => TokenType::Not,
            "null" => TokenType::Null,
            "in" => TokenType::In,
            "any" => TokenType::Any,
            "all" => TokenType::All,
            "exists" => TokenType::Exists,

            // must be table or col reference?
            _ => TokenType::Identifier,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    lexeme: String,
    type_: TokenType,
    indices: (usize, usize),
}

impl Token {
    pub fn new(lexeme: &[char], type_: TokenType, indices: (usize, usize)) -> Self {
        Self {
            lexeme: lexeme.iter().cloned().collect(),
            type_,
            indices,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token(lexeme='{}', type={:?}, indices=[{},{}])",
            self.lexeme, self.type_, self.indices.0, self.indices.1
        )
    }
}

#[derive(Clone, Debug, Default)]
struct Scanner<'a> {
    source: &'a [char],
    l: usize,
    r: usize,
    buffer: Vec<Token>,
}

impl<'a> Scanner<'a> {
    /// Creates a new [`Scanner`] from the raw sql string.
    pub fn new(source: &'a [char]) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }

    fn is_at_end(&self) -> bool {
        self.r >= self.source.len()
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

    fn add_token(&mut self, type_: TokenType) {
        let tok = Token::new(&self.source[self.l..self.r], type_, (self.l, self.r));
        self.buffer.push(tok);
    }

    fn is_next_char(&mut self, expected: char) -> bool {
        if self.source[self.r] != expected {
            return false;
        };
        self.r += 1;
        true
    }

    pub fn scan_tokens(mut self) -> impl IntoIterator<Item = Token> {
        while !self.is_at_end() {
            let c = self.source[self.r];
            self.l = self.r;
            self.r += 1;

            match c {
                '(' => self.add_token(TokenType::LeftParen),
                ')' => self.add_token(TokenType::RightParen),
                ',' => self.add_token(TokenType::Comma),
                ';' => self.add_token(TokenType::Semicolon),
                '-' => self.add_token(TokenType::Minus),
                '+' => self.add_token(TokenType::Plus),
                '*' => self.add_token(TokenType::Star),
                '=' => self.add_token(TokenType::Eq),
                '!' => {
                    let tt = match self.is_next_char('=') {
                        true => TokenType::Neq,
                        false => TokenType::Not,
                    };
                    self.add_token(tt);
                }
                '<' => {
                    let tt = match self.is_next_char('=') {
                        true => TokenType::LtEq,
                        false => TokenType::Lt,
                    };
                    self.add_token(tt);
                }
                '>' => {
                    let tt = match self.is_next_char('=') {
                        true => TokenType::GtEq,
                        false => TokenType::Gt,
                    };
                    self.add_token(tt);
                }
                // nop
                ' ' | '\n' | '\r' | '\t' => {}
                'A'..='Z' | 'a'..='z' | '_' => {
                    while self.is_alpha_numeric(self.peek()) {
                        self.r += 1;
                    }

                    let tt = TokenType::from(&self.source[self.l..self.r]);
                    self.add_token(tt);
                }
                _ => unreachable!("i dont recnogize this char {}", c),
            }
        }

        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner() {
        let query = r#"
        SELECT
            employee_id,
            month,
            avg(salary) AS avg_salary,
            min(salary) AS min_salary,
            max(salary) AS max_salary
        FROM employees
        GROUP BY
            employee_id, month
        ORDER BY
            month ASC, avg_salary DESC;
        "#;
        let raw_query = &query.chars().collect::<Vec<char>>();
        let scanner = Scanner::new(raw_query);
        let tokens = scanner.scan_tokens();
        for token in tokens.into_iter() {
            println!("{}", token);
        }
    }
}
