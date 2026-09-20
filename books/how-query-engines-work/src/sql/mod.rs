use std::{collections::HashSet, sync::Arc};

use crate::{
    data_source::registry::SourceRegistry,
    dataframe::DataFrame,
    logical::{
        expr::{
            LogicalExpr, LogicalExprKind, alias, and, avg, col, col_idx, eq, gt, gteq, lt, lteq,
            max, min, neq, or, sum,
        },
        plan::LogicalPlan,
    },
};

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
pub struct Scanner<'a> {
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
    pub ident: String,
    pub args: Vec<SqlExpr>,
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
    pub expr: SqlExpr,
    pub alias: SqlIdentifier,
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
    pub expr: SqlExpr,
    pub asc: bool,
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

    // this is the logical plan
    pub fn as_dataframe(&self, sources: Arc<SourceRegistry>) -> DataFrame {
        let select = match self.kind() {
            SqlExprKind::Select(select) => select.clone(),
            _ => panic!("Expected select expr, found {}", self),
        };

        let table_id = sources.id_by_name(&select.table_name).unwrap_or_else(|| {
            panic!(
                "Table '{}' not found in registered sources",
                select.table_name
            );
        });

        let data_source = sources.get(table_id).unwrap();

        let mut plan = DataFrame::new(
            LogicalPlan::scan(
                table_id,
                select.clone().table_name,
                data_source.schema().clone(),
            ),
            sources.clone(),
        );

        let projection_expr = select
            .clone()
            .projection
            .into_iter()
            .map(|e| self.create_logical_expr(e, plan.clone()))
            .collect::<Vec<LogicalExpr>>();
        let aggexprcount: u8 = projection_expr
            .iter()
            .map(|e| self.is_agg_expr(e.clone()))
            .sum();

        if aggexprcount == 0 && select.clone().group_by.map(|v| v.len()).unwrap_or(0) > 0 {
            panic!("GROUP BY without aggregate expressions is not supported");
        }

        let colnames_in_proj = self.get_referenced_cols(projection_expr.clone());
        let colnames_in_pred = self.get_cols_referenced_by_predicate(select.clone(), plan.clone());

        if aggexprcount == 0 {
            return self.create_non_agg_query(
                select,
                plan,
                projection_expr,
                colnames_in_pred,
                colnames_in_proj,
            );
        }

        let mut proj = Vec::new();
        let mut aggs = Vec::new();

        let ngroupcols = select.clone().group_by.map(|v| v.len()).unwrap_or(0);
        let mut groupcount = 0;

        projection_expr.iter().for_each(|e| {
            match e.kind() {
                LogicalExprKind::Aggregate(_) => {
                    proj.push(col_idx(ngroupcols + aggs.len()));
                    aggs.push(e.clone());
                }
                LogicalExprKind::Alias(aliase) => {
                    let innerxp = aliase.expr.clone();
                    if !matches!(innerxp.kind(), LogicalExprKind::Aggregate(_)) {
                        panic!(
                            "Alias in aggregate query must wrap an aggregate expression, found: {}",
                            innerxp
                        );
                    };
                    proj.push(alias(col_idx(ngroupcols + aggs.len()), aliase.name.clone()));
                    aggs.push(innerxp);
                }
                _ => {
                    proj.push(col_idx(groupcount));
                    groupcount += 1;
                }
            };
        });

        plan = self.plan_aggregate_query(
            projection_expr,
            select.clone(),
            colnames_in_pred,
            plan,
            aggs,
        );
        plan = plan.project(proj);
        if let Some(having) = select.having {
            plan = plan.filter(self.create_logical_expr(having, plan.clone()));
        }

        if let Some(limit) = select.limit {
            plan = plan.limit(limit);
        }

        plan
    }

    fn plan_aggregate_query(
        &self,
        proj_exprs: Vec<LogicalExpr>,
        select: SqlSelect,
        colnames_in_pred: HashSet<String>,
        df: DataFrame,
        agg_exprs: Vec<LogicalExpr>,
    ) -> DataFrame {
        let mut plan = df;
        let projwithoutaggs = proj_exprs
            .into_iter()
            .filter(|e| self.is_agg_expr(e.clone()) == 0)
            .collect::<Vec<LogicalExpr>>();

        let colnames_in_agg = self.get_referenced_cols(agg_exprs.clone());
        if let Some(pred) = select.predicate {
            let colnamesinprojwithoutaggs = self.get_referenced_cols(projwithoutaggs.clone());
            let allrequiredcols: HashSet<String> = colnamesinprojwithoutaggs
                .union(&colnames_in_pred)
                .map(|s| s.to_owned())
                .collect::<HashSet<String>>()
                .union(&colnames_in_agg)
                .map(|s| s.to_owned())
                .collect();
            let missing = allrequiredcols
                .difference(&colnamesinprojwithoutaggs)
                .map(|s| s.to_owned())
                .collect::<HashSet<String>>();

            if missing.is_empty() {
                plan = plan.project(projwithoutaggs);
            } else {
                let haha = missing.iter().map(|s| col(s)).collect::<Vec<LogicalExpr>>();
                let mut aaa = projwithoutaggs.clone();
                aaa.extend_from_slice(&haha);
                plan = plan.project(aaa);
            }
            plan = plan.filter(self.create_logical_expr(pred, plan.clone()));
        };

        let groupbyexpr = select
            .group_by
            .unwrap()
            .iter()
            .map(|e| self.create_logical_expr(e.clone(), plan.clone()))
            .collect::<Vec<LogicalExpr>>();
        plan.agg(groupbyexpr, agg_exprs)
    }

    fn create_non_agg_query(
        &self,
        select: SqlSelect,
        df: DataFrame,
        proj_expr: Vec<LogicalExpr>,
        pred_colnames: HashSet<String>,
        proj_colnames: HashSet<String>,
    ) -> DataFrame {
        let mut plan = df;
        if select.predicate.is_none() {
            plan = plan.project(proj_expr);
            if let Some(limit) = select.limit {
                plan.limit(limit);
            }
            return plan;
        }

        let missing: HashSet<String> = pred_colnames
            .difference(&proj_colnames)
            .map(|xd| xd.clone())
            .collect();
        if missing.is_empty() {
            plan = plan.project(proj_expr);
            plan = plan.filter(self.create_logical_expr(select.predicate.unwrap(), plan.clone()));
        } else {
            // because the filter references some columns that are not in the projection output we
            // need to create an interim projection that has the additional columns and then we
            // need to remove them after the filter has been applied
            let n = proj_expr.len();
            let kebab = missing.iter().map(|s| col(s)).collect::<Vec<LogicalExpr>>();
            let mut interim_proj = proj_expr.clone();
            interim_proj.extend_from_slice(&kebab);

            plan = plan.project(interim_proj);
            plan = plan.filter(self.create_logical_expr(select.predicate.unwrap(), plan.clone()));

            let exprs = (0..n)
                .map(|i| col(plan.logical_plan().schema().fields()[i].name()))
                .collect::<Vec<LogicalExpr>>();
            plan = plan.project(exprs);
        }

        if let Some(limit) = select.limit {
            plan.limit(limit);
        }

        plan
    }

    fn get_cols_referenced_by_predicate(
        &self,
        select: SqlSelect,
        plan: DataFrame,
    ) -> HashSet<String> {
        let mut acc = HashSet::new();
        if select.predicate.is_none() {
            return acc;
        }
        let pred = select.predicate.unwrap();
        let expr = self.create_logical_expr(pred, plan.clone());
        self.visit(expr, &mut acc);
        let validcolnames = plan
            .logical_plan()
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect::<HashSet<String>>();
        let mut xdd = acc.iter().map(|s| s.clone()).collect::<HashSet<String>>();
        let copied = xdd.clone();
        for xd in copied {
            if !validcolnames.contains(&xd) {
                xdd.remove(&xd);
            }
        }
        xdd
    }

    fn is_agg_expr(&self, e: LogicalExpr) -> u8 {
        match e.kind() {
            crate::logical::expr::LogicalExprKind::Aggregate(_) => 1,
            crate::logical::expr::LogicalExprKind::Alias(alias_logical_expr) => {
                if matches!(
                    alias_logical_expr.expr.kind(),
                    LogicalExprKind::Aggregate(_)
                ) {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    fn get_referenced_cols(&self, exprs: Vec<LogicalExpr>) -> HashSet<String> {
        let mut xd = HashSet::new();
        exprs.iter().for_each(|e| self.visit(e.clone(), &mut xd));
        xd
    }

    fn visit(&self, expr: LogicalExpr, acc: &mut HashSet<String>) {
        match expr.kind() {
            crate::logical::expr::LogicalExprKind::Column(e) => {
                acc.insert(e.name().to_owned());
            }
            crate::logical::expr::LogicalExprKind::Binary(e) => {
                self.visit(e.left().clone(), acc);
                self.visit(e.right().clone(), acc);
            }
            crate::logical::expr::LogicalExprKind::Aggregate(e) => {
                self.visit(e.expr.clone(), acc);
            }
            crate::logical::expr::LogicalExprKind::Alias(e) => {
                self.visit(e.expr.clone(), acc);
            }
            _ => panic!("Unexpected expr in projection {}", expr),
        };
    }

    fn create_logical_expr(&self, expr: SqlExpr, input: DataFrame) -> LogicalExpr {
        match expr.kind() {
            SqlExprKind::Identifier(sql) => col(sql.ident.clone()),
            SqlExprKind::Alias(sql) => alias(
                self.create_logical_expr(sql.expr.clone(), input),
                sql.alias.ident.clone(),
            ),
            SqlExprKind::Binary(sql) => {
                let l = self.create_logical_expr(sql.l.clone(), input.clone());
                let r = self.create_logical_expr(sql.r.clone(), input.clone());
                match sql.op.to_lowercase().as_str() {
                    "=" => eq(l, r),
                    "!=" => neq(l, r),
                    ">" => gt(l, r),
                    ">=" => gteq(l, r),
                    "<" => lt(l, r),
                    "<=" => lteq(l, r),
                    "and" => and(l, r),
                    "or" => or(l, r),
                    "+" => l + r,
                    "-" => l - r,
                    "*" => l * r,
                    "/" => l / r,
                    _ => panic!("Invalid operator {}", sql.op),
                }
            }
            SqlExprKind::Function(sql) => match sql.ident.to_lowercase().as_str() {
                "min" | "max" | "sum" | "avg" => {
                    if sql.args.is_empty() {
                        panic!("{} requires one argument", sql.ident.to_uppercase());
                    }

                    let arg = self.create_logical_expr(sql.args.first().unwrap().clone(), input);
                    match sql.ident.to_lowercase().as_str() {
                        "min" => min(arg),
                        "max" => max(arg),
                        "avg" => avg(arg),
                        "sum" => sum(arg),
                        _ => panic!("Unsupported aggregate function: {}", sql.ident),
                    }
                }
                _ => panic!("Unsupported aggregate function: {}", sql.ident),
            },
            _ => panic!("Cannot create logical expression from sql: {}", expr),
        }
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
pub struct Parser {
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
        let mut exprs = Vec::new();

        loop {
            let Some(expr) = self.parse_expr() else {
                break;
            };

            exprs.push(expr);

            if !self.stream.consume_token_type(TokenType::Comma) {
                break;
            }
        }

        exprs
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

        while let Some(s) = sort {
            let s = match s.kind() {
                SqlExprKind::Identifier(_) => SqlSort::new(s, true).into(),
                SqlExprKind::Sort(_) => s,
                _ => panic!("Unexpected expression {} after order by", s),
            };

            sortlist.push(s);

            if matches!(
                self.stream.peek(),
                Some(token) if token.type_ == TokenType::Comma
            ) {
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
        let tokens = scanner.scan_tokens().into_iter().collect::<Vec<Token>>();
        let xd = tokens.clone();

        let mut parser = Parser::new(tokens);
        let ast = parser.parse();
        let SqlExprKind::Select(select) = ast.kind() else {
            panic!("Expected SELECT");
        };

        assert_eq!(select.projection.len(), 5);
        assert_eq!(select.table_name, "employees");
        assert_eq!(select.group_by.as_ref().unwrap().len(), 2);
        assert_eq!(select.order_by.as_ref().unwrap().len(), 2);

        for xdd in xd {
            println!("{}", xdd);
        }
    }
}
