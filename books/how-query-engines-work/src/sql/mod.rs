// TODO: implement SQL xd
#[allow(dead_code)]
pub enum TokenType {
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
}

#[allow(dead_code)]
pub struct Token {
    text: String,
    type_: TokenType,
    end_offset: usize,
}
