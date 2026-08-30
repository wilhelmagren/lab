// TODO: implement SQL xd
enum TokenType {}

struct Token {
    text: String,
    type_: TokenType,
    end_offset: usize,
}
