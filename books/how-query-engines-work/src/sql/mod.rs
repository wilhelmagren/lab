use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
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
        let mut line = 0;
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
                ' ' | '\r' | '\t' => {}
                '\n' => {
                    line += 1;
                }
                'A'..='Z' | 'a'..='z' | '_' => {
                    while self.is_alpha_numeric(self.peek()) {
                        self.r += 1;
                    }

                    let tt = TokenType::from(&self.source[self.l..self.r]);
                    self.add_token(tt);
                }
                _ => panic!(
                    "unexpected character '{}' (line={}, indices=[{},{}])",
                    c, line, self.l, self.r
                ),
            }
        }

        self.buffer
    }
}

#[derive(Clone, Debug)]
pub enum SqlExprKind {
    Identifier(SqlIdentifier),
    Alias(SqlAlias),
    Binary(SqlBinaryExpr),
    Select(SqlSelect),
    Sort(SqlSort),
    Function(SqlFunction),
}

#[derive(Clone, Debug)]
struct SqlSelect {
    pub projection: Vec<SqlExpr>,
    pub table_name: String,
    pub predicate: Option<SqlExpr>,
    pub group_by: Option<Vec<SqlExpr>>,
    pub having: Option<SqlExpr>,
    pub order_by: Option<Vec<SqlExpr>>,
    pub limit: Option<usize>,
}

impl SqlSelect {
    pub fn new(
        projection: Vec<SqlExpr>,
        table_name: String,
        predicate: Option<SqlExpr>,
        group_by: Option<Vec<SqlExpr>>,
        having: Option<SqlExpr>,
        order_by: Option<Vec<SqlExpr>>,
        limit: Option<usize>,
    ) -> Self {
        Self {
            projection,
            table_name,
            predicate,
            group_by,
            having,
            order_by,
            limit,
        }
    }
}

impl Into<SqlExpr> for SqlSelect {
    fn into(self) -> SqlExpr {
        SqlExpr(Arc::new(SqlExprKind::Select(self)))
    }
}

impl std::fmt::Display for SqlSelect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TODO: SELECT DISPLAY")
    }
}

#[derive(Clone, Debug)]
pub struct SqlBinaryExpr {
    l: SqlExpr,
    op: String,
    r: SqlExpr,
}

impl SqlBinaryExpr {
    pub fn new(l: SqlExpr, op: impl Into<String>, r: SqlExpr) -> Self {
        Self {
            l,
            op: op.into(),
            r,
        }
    }
}

impl Into<SqlExpr> for SqlBinaryExpr {
    fn into(self) -> SqlExpr {
        SqlExpr(Arc::new(SqlExprKind::Binary(self)))
    }
}

impl std::fmt::Display for SqlBinaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.l, self.op, self.r)
    }
}

#[derive(Clone, Debug)]
pub struct SqlFunction {
    ident: String,
    args: Vec<SqlExpr>,
}

impl SqlFunction {
    pub fn new(ident: String, args: Vec<SqlExpr>) -> Self {
        Self { ident, args }
    }
}

impl Into<SqlExpr> for SqlFunction {
    fn into(self) -> SqlExpr {
        SqlExpr(Arc::new(SqlExprKind::Function(self)))
    }
}

impl std::fmt::Display for SqlFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:?})", self.ident, self.args)
    }
}

#[derive(Clone, Debug)]
pub struct SqlAlias {
    expr: SqlExpr,
    alias: SqlIdentifier,
}

impl SqlAlias {
    pub fn new(expr: SqlExpr, alias: SqlIdentifier) -> Self {
        Self { expr, alias }
    }
}

impl Into<SqlExpr> for SqlAlias {
    fn into(self) -> SqlExpr {
        SqlExpr(Arc::new(SqlExprKind::Alias(self)))
    }
}

impl std::fmt::Display for SqlAlias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.expr, self.alias)
    }
}

#[derive(Clone, Debug)]
pub struct SqlSort {
    expr: SqlExpr,
    asc: bool,
}

impl SqlSort {
    pub fn new(expr: SqlExpr, asc: bool) -> Self {
        Self { expr, asc }
    }
}

impl Into<SqlExpr> for SqlSort {
    fn into(self) -> SqlExpr {
        SqlExpr(Arc::new(SqlExprKind::Sort(self)))
    }
}

impl std::fmt::Display for SqlSort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.asc {
            write!(f, "{} ASC", self.expr)
        } else {
            write!(f, "{} DESC", self.expr)
        }
    }
}

#[derive(Clone, Debug)]
pub struct SqlIdentifier {
    ident: String,
}

impl Into<SqlExpr> for SqlIdentifier {
    fn into(self) -> SqlExpr {
        SqlExpr(Arc::new(SqlExprKind::Identifier(self)))
    }
}

impl SqlIdentifier {
    pub fn new(ident: impl Into<String>) -> Self {
        Self {
            ident: ident.into(),
        }
    }
}

impl std::fmt::Display for SqlIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ident)
    }
}

#[derive(Clone, Debug)]
pub struct SqlExpr(Arc<SqlExprKind>);

impl SqlExpr {
    pub fn kind(&self) -> &SqlExprKind {
        self.0.as_ref()
    }
}

impl std::fmt::Display for SqlExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            SqlExprKind::Identifier(sql) => write!(f, "{}", sql),
            SqlExprKind::Alias(sql) => write!(f, "{}", sql),
            SqlExprKind::Binary(sql) => write!(f, "{}", sql),
            SqlExprKind::Select(sql) => write!(f, "{}", sql),
            SqlExprKind::Sort(sql) => write!(f, "{}", sql),
            SqlExprKind::Function(sql) => write!(f, "{}", sql),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TokenStream {
    tokens: Vec<Token>,
    i: usize,
}

impl TokenStream {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            ..Default::default()
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.i < self.tokens.len() {
            return Some(&self.tokens[self.i]);
        }

        None
    }

    pub fn next(&mut self) -> Option<&Token> {
        if self.i < self.tokens.len() {
            self.i += 1;
            return Some(&self.tokens[self.i - 1]);
        }

        None
    }

    pub fn consume_keywords(&mut self, keywords: Vec<String>) -> bool {
        for kw in keywords {
            if !self.consume_keyword(kw) {
                return false;
            }
        }
        true
    }

    pub fn consume_keyword(&mut self, kw: String) -> bool {
        if let Some(tok) = self.peek() {
            match tok.type_ {
                TokenType::Select
                | TokenType::From
                | TokenType::Where
                | TokenType::Limit
                | TokenType::As
                | TokenType::Left
                | TokenType::Right
                | TokenType::Inner
                | TokenType::Join
                | TokenType::Group
                | TokenType::Order
                | TokenType::By
                | TokenType::Asc
                | TokenType::Desc
                | TokenType::Explain
                | TokenType::Avg
                | TokenType::Min
                | TokenType::Max
                | TokenType::Sum => {
                    if tok.lexeme.to_lowercase() == kw.to_lowercase() {
                        self.i += 1;
                        return true;
                    }
                }
                _ => {}
            }
        };
        false
    }

    pub fn consume_token_type(&mut self, t: TokenType) -> bool {
        if let Some(tok) = self.peek() {
            if tok.type_ == t {
                self.i += 1;
                return true;
            }
        };
        false
    }
}

#[derive(Clone, Debug)]
struct Parser {
    stream: TokenStream,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: TokenStream::new(tokens),
        }
    }

    fn next_precedence(&mut self) -> usize {
        let token = match self.stream.peek() {
            Some(tok) => tok,
            None => {
                return 0;
            }
        };

        let precedence = match token.type_ {
            TokenType::As | TokenType::Asc | TokenType::Desc => 10,
            TokenType::Or => 20,
            TokenType::And => 30,

            TokenType::Lt
            | TokenType::LtEq
            | TokenType::Eq
            | TokenType::Neq
            | TokenType::Gt
            | TokenType::GtEq => 40,
            TokenType::Plus | TokenType::Minus => 50,
            TokenType::Star | TokenType::Slash => 60,
            TokenType::LeftParen => 70,
            _ => 0,
        };
        precedence
    }

    fn parse_prefix(&mut self) -> Option<SqlExpr> {
        let token = match self.stream.next() {
            Some(tok) => tok,
            None => {
                return None;
            }
        };

        let expr: SqlExpr = match token.type_ {
            TokenType::Select => self.parse_select(),
            TokenType::Min
            | TokenType::Max
            | TokenType::Sum
            | TokenType::Avg
            | TokenType::Identifier => SqlIdentifier::new(token.lexeme.as_str()).into(),
            TokenType::LeftParen => {
                let expr = match self.parse_expr() {
                    Some(e) => e,
                    None => panic!("Expected expression after '('"),
                };
                if !self.stream.consume_token_type(TokenType::RightParen) {
                    panic!("Expected ')' after expression");
                };
                expr
            }
            _ => panic!("Unexpected token '{}'", token),
        };
        Some(expr)
    }

    fn parse_expr(&mut self) -> Option<SqlExpr> {
        self.parse_with_precedence(0)
    }

    fn parse_infix(&mut self, left: SqlExpr, precedence: usize) -> SqlExpr {
        let token = match self.stream.peek() {
            Some(tok) => tok.clone(),
            None => {
                panic!("Expected a token in the infix... uh oh stinky");
            }
        };

        let expr: SqlExpr = match token.type_ {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::Eq
            | TokenType::Gt
            | TokenType::Lt
            | TokenType::GtEq
            | TokenType::LtEq
            | TokenType::Neq => {
                // consume the token
                self.stream.next();
                SqlBinaryExpr::new(
                    left,
                    token.lexeme.clone(),
                    self.parse_with_precedence(precedence).unwrap(),
                )
                .into()
            }

            // keywords
            TokenType::As => {
                // consume the token
                self.stream.next();
                SqlAlias::new(left, self.parse_identifier()).into()
            }
            TokenType::And | TokenType::Or => {
                // consume the token
                self.stream.next();
                SqlBinaryExpr::new(
                    left,
                    token.lexeme.clone(),
                    self.parse_with_precedence(precedence).unwrap(),
                )
                .into()
            }
            TokenType::Asc | TokenType::Desc => {
                // consume the token
                self.stream.next();
                SqlSort::new(left, token.type_ == TokenType::Asc).into()
            }
            TokenType::LeftParen => {
                match left.kind() {
                    SqlExprKind::Identifier(ident_sql) => {
                        // consume the token
                        self.stream.next();
                        // there has to exist another token here
                        let args = match self.stream.peek().unwrap().type_ {
                            TokenType::RightParen => Vec::new(),
                            _ => self.parse_expr_list(),
                        };
                        if self.stream.next().unwrap().type_ != TokenType::RightParen {
                            panic!("Expected ')' after function args");
                        }
                        SqlFunction::new(ident_sql.ident.clone(), args).into()
                    }
                    _ => panic!("Unexpected LPAREN"),
                }
            }
            _ => panic!("Unexpected infix token {}", token),
        };

        expr
    }

    fn parse_expr_list(&mut self) -> Vec<SqlExpr> {
        let mut v = Vec::new();
        let mut expr = self.parse_expr();
        while !expr.is_none() {
            v.push(expr.unwrap());
            if self.stream.peek().unwrap().type_ == TokenType::Comma {
                self.stream.next();
            } else {
                break;
            }
            expr = self.parse_expr();
        }
        v
    }

    fn parse_with_precedence(&mut self, precedence: usize) -> Option<SqlExpr> {
        let maybe_expr = self.parse_prefix();
        if maybe_expr.is_none() {
            return None;
        };
        let mut expr = maybe_expr.unwrap();
        while precedence < self.next_precedence() {
            let np = self.next_precedence();
            expr = self.parse_infix(expr, np);
        }
        Some(expr)
    }

    fn parse_identifier(&mut self) -> SqlIdentifier {
        let expr = match self.parse_expr() {
            Some(e) => e,
            None => panic!("Expected identifier, found EOF"),
        };
        match expr.kind() {
            SqlExprKind::Identifier(ident) => ident.clone(),
            _ => panic!("Expected identifier, found {}", expr),
        }
    }

    fn parse_select(&mut self) -> SqlExpr {
        let projection = self.parse_expr_list();
        if self.stream.consume_keyword("FROM".into()) {
            let table_expr = self.parse_expr().expect("Expected table name after FROM");
            let table_name = match table_expr.kind() {
                SqlExprKind::Identifier(xd) => xd.ident.clone(),
                _ => panic!("Expected table name after FROM, found {}", table_expr),
            };

            let mut filter_expr = None;
            if self.stream.consume_keyword("WHERE".into()) {
                filter_expr = self.parse_expr();
            }

            let mut group_by = None;
            if self
                .stream
                .consume_keywords(vec!["GROUP".to_owned(), "BY".to_owned()])
            {
                group_by = Some(self.parse_expr_list());
            }

            let mut having_expr = None;
            if self.stream.consume_keyword("HAVING".into()) {
                having_expr = self.parse_expr();
            }

            let mut order_by = None;
            if self
                .stream
                .consume_keywords(vec!["ORDER".to_owned(), "BY".to_owned()])
            {
                order_by = Some(self.parse_order());
            }

            let mut limit = None;
            if self.stream.consume_keyword("LIMIT".into()) {
                let limit_expr = self.parse_expr().unwrap();
                todo!()
            }

            return SqlSelect::new(
                projection,
                table_name,
                filter_expr,
                group_by,
                having_expr,
                order_by,
                limit,
            )
            .into();
        } else {
            panic!("Expected FROM keyword, found {:?}", self.stream.peek());
        };
    }

    fn parse_order(&mut self) -> Vec<SqlExpr> {
        let mut sortlist = Vec::new();
        let mut sort = self.parse_expr();
        while !sort.is_none() {
            sort = match sort {
                Some(s) => match s.kind() {
                    SqlExprKind::Identifier(_) => Some(SqlSort::new(s, true).into()),
                    SqlExprKind::Sort(_) => Some(s),
                    _ => panic!("Unexpected expression {} after order by", s),
                },
                None => unreachable!(),
            };
            sortlist.push(sort.unwrap());
            if self.stream.peek().unwrap().type_ == TokenType::Comma {
                self.stream.next();
            } else {
                break;
            }
            sort = self.parse_expr();
        }
        sortlist
    }

    pub fn parse(&mut self) -> SqlExpr {
        self.parse_with_precedence(0).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pratt_parser() {
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
        let mut parser = Parser::new(tokens.into_iter().collect::<Vec<Token>>());
        let ast = parser.parse();
        let SqlExprKind::Select(select) = ast.kind() else {
            panic!("Expected SELECT");
        };

        assert_eq!(select.projection.len(), 5);
        assert_eq!(select.table_name, "employees");
        assert_eq!(select.group_by.as_ref().unwrap().len(), 2);
        assert_eq!(select.order_by.as_ref().unwrap().len(), 2);
    }
}
