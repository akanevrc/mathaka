use std::iter::Peekable;
use anyhow::{
    Error,
    Result,
};
use crate::data::*;
use crate::bail_info;

pub fn lex<'input>(input: &'input str) -> Result<Vec<TokenInfo<'input>>, Vec<Error>> {
    let mut tokens = Vec::new();
    let mut errs = Vec::new();
    let mut chars = CharInfos::new(input).peekable();
    loop {
        match assume(&mut chars) {
            Ok(Some(token @ TokenInfo(Token::Eof, _))) => {
                tokens.push(token);
                break;
            },
            Ok(Some(token)) =>
                tokens.push(token),
            Ok(None) =>
                (),
            Err(e) => {
                errs.push(e);
                chars.next();
            },
        }
    }
    if errs.len() == 0 {
        Ok(tokens)
    }
    else {
        Err(errs)
    }
}

fn assume<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if let Some(token) = assume_eof(chars)? {
        Ok(Some(token))
    }
    else if let Some(_) = assume_whitespace(chars)? {
        Ok(None)
    }
    else if let Some(token) = assume_token(chars)? {
        Ok(Some(token))
    }
    else {
        let (info, c) = &chars.peek().unwrap();
        bail_info!(info, "Invalid token found: `{}`", c);
    }
}

fn assume_eof<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if let None = chars.peek() {
        Ok(Some(TokenInfo(Token::Eof, StrInfo::eof())))
    }
    else {
        Ok(None)
    }
}

fn assume_whitespace<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<()>> {
    let mut consumed = false;
    while map_char_info(chars.peek(), false, is_whitespace) {
        chars.next();
        consumed = true;
    }
    if consumed {
        Ok(Some(()))
    }
    else {
        Ok(None)
    }
}

fn assume_token<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if let Some(token) = assume_semicolon(chars)? {
        Ok(Some(token))
    }
    else if let Some(token) = assume_keyword_or_ident(chars)? {
        Ok(Some(token))
    }
    else if let Some(token) = assume_num(chars)? {
        Ok(Some(token))
    }
    else if let Some(token) = assume_comma(chars)? {
        Ok(Some(token))
    }
    else if let Some(token) = assume_paren(chars)? {
        Ok(Some(token))
    }
    else if let Some(token) = assume_symbol_or_op_code(chars)? {
        Ok(Some(token))
    }
    else {
        Ok(None)
    }
}

fn assume_semicolon<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if map_char_info(chars.peek(), false, is_semicolon) {
        let (info, _) = chars.next().unwrap();
        Ok(Some(TokenInfo(Token::Semicolon, info)))
    }
    else {
        Ok(None)
    }
}

fn assume_keyword_or_ident<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if map_char_info(chars.peek(), false, is_ident_head) {
        let (info_head, c) = chars.next().unwrap();
        let mut token = String::from(c);
        let mut info_tail = info_head.clone();
        while map_char_info(chars.peek(), false, is_ident_tail) {
            let (info, c) = chars.next().unwrap();
            token.push(c);
            info_tail = info;
        }
        let info = info_head.extend(&info_tail);
        Ok(Some(TokenInfo(Token::Ident(token), info)))
    }
    else {
        Ok(None)
    }
}

fn assume_num<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if map_char_info(chars.peek(), false, is_num) {
        let (info_head, c) = chars.next().unwrap();
        let mut token = String::from(c);
        let mut info_tail = info_head.clone();
        while map_char_info(chars.peek(), false, is_num) {
            let (info, c) = chars.next().unwrap();
            token.push(c);
            info_tail = info;
        }
        if map_char_info(chars.peek(), false, is_dot) {
            let (info, dot) = chars.next().unwrap();
            token.push(dot);
            info_tail = info;
            while map_char_info(chars.peek(), false, is_num) {
                let (info, c) = chars.next().unwrap();
                token.push(c);
                info_tail = info;
            }
            let info = info_head.extend(&info_tail);
            Ok(Some(TokenInfo(Token::Num(token), info)))
        }
        else {
            let info = info_head.extend(&info_tail);
            Ok(Some(TokenInfo(Token::Num(token), info)))
        }
    }
    else {
        Ok(None)
    }
}

fn assume_comma<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    let c = chars.peek();
    if map_char_info(c, false, is_comma) {
        let (info, _) = chars.next().unwrap();
        Ok(Some(TokenInfo(Token::Comma, info)))
    }
    else {
        Ok(None)
    }
}

fn assume_paren<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    let c = chars.peek();
    if map_char_info(c, false, is_l_brace) {
        let (info, _) = chars.next().unwrap();
        Ok(Some(TokenInfo(Token::LBrace, info)))
    }
    else if map_char_info(c, false, is_r_brace) {
        let (info, _) = chars.next().unwrap();
        Ok(Some(TokenInfo(Token::RBrace, info)))
    }
    else if map_char_info(c, false, is_l_paren) {
        let (info, _) = chars.next().unwrap();
        Ok(Some(TokenInfo(Token::LParen, info)))
    }
    else if map_char_info(c, false, is_r_paren) {
        let (info, _) = chars.next().unwrap();
        Ok(Some(TokenInfo(Token::RParen, info)))
    }
    else {
        Ok(None)
    }
}

fn assume_symbol_or_op_code<'input>(chars: &mut Peekable<CharInfos<'input>>) -> Result<Option<TokenInfo<'input>>> {
    if map_char_info(chars.peek(), false, is_op_code) {
        let (info_head, c) = chars.next().unwrap();
        let mut token = String::from(c);
        let mut info_tail = info_head.clone();
        while map_char_info(chars.peek(), false, is_op_code) {
            let (info, c) = chars.next().unwrap();
            token.push(c);
            info_tail = info;
        }
        let info = info_head.extend(&info_tail);
        if is_colon_eq(&token) {
            Ok(Some(TokenInfo(Token::ColonEq, info)))
        }
        else {
            Ok(Some(TokenInfo(Token::OpCode(token), info)))
        }
    }
    else {
        Ok(None)
    }
}

fn map_char_info<'input, T>(info: Option<&(StrInfo<'input>, char)>, default: T, f: fn(&char) -> T) -> T {
    match info {
        Some((_, c)) => f(c),
        None => default,
    }
}

fn is_whitespace(c: &char) -> bool {
    c.is_ascii_whitespace()
}

fn is_semicolon(c: &char) -> bool {
    *c == ';'
}

fn is_dot(c: &char) -> bool {
    *c == '.'
}

fn is_ident_head(c: &char) -> bool {
    c.is_ascii_alphabetic()
}

fn is_ident_tail(c: &char) -> bool {
    *c == '_' || c.is_ascii_alphanumeric()
}

fn is_num(c: &char) -> bool {
    c.is_ascii_digit()
}

fn is_op_code(c: &char) -> bool {
    [
        '!',
        '#',
        '$',
        '%',
        '&',
        '*',
        '+',
        '.',
        '/',
        '<',
        '=',
        '>',
        '?',
        '@',
        '\\',
        '^',
        '|',
        '-',
        '~',
        ':',
    ].contains(c)
}

fn is_comma(c: &char) -> bool {
    *c == ','
}

fn is_l_brace(c: &char) -> bool {
    *c == '{'
}

fn is_r_brace(c: &char) -> bool {
    *c == '}'
}

fn is_l_paren(c: &char) -> bool {
    *c == '('
}

fn is_r_paren(c: &char) -> bool {
    *c == ')'
}

fn is_colon_eq(s: &str) -> bool {
    s == ":="
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_to_tokens(input: &str) -> Vec<Token> {
        super::lex(input).unwrap().into_iter().map(|TokenInfo(token, _)| token).collect()
    }

    fn lex_to_infos(input: &str) -> Vec<StrInfo> {
        super::lex(input).unwrap().into_iter().map(|TokenInfo(_, info)| info).collect()
    }

    fn lex_to_err_msgs(input: &str) -> Vec<String> {
        super::lex(input).err().unwrap().into_iter().map(|e| e.to_string()).collect()
    }

    #[test]
    fn test_lex_eof() {
        assert_eq!(lex_to_tokens(""), &[Token::Eof]);
    }

    #[test]
    fn test_lex_whitespace() {
        assert_eq!(lex_to_tokens(" \t\n\r"), &[Token::Eof]);
    }

    #[test]
    fn test_lex_semicolon() {
        assert_eq!(lex_to_tokens(";"), &[Token::Semicolon, Token::Eof]);
    }

    #[test]
    fn test_lex_ident() {
        assert_eq!(lex_to_tokens("A"), &[Token::Ident("A".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("A0"), &[Token::Ident("A0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("A_0"), &[Token::Ident("A_0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("A0_"), &[Token::Ident("A0_".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("Aa"), &[Token::Ident("Aa".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("a"), &[Token::Ident("a".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("a0"), &[Token::Ident("a0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("a_0"), &[Token::Ident("a_0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("a0_"), &[Token::Ident("a0_".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("aa"), &[Token::Ident("aa".to_string()), Token::Eof]);
    }

    #[test]
    fn test_lex_num() {
        assert_eq!(lex_to_tokens("0"), &[Token::Num("0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("1"), &[Token::Num("1".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("2"), &[Token::Num("2".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("3"), &[Token::Num("3".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("4"), &[Token::Num("4".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("5"), &[Token::Num("5".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("6"), &[Token::Num("6".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("7"), &[Token::Num("7".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("8"), &[Token::Num("8".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("9"), &[Token::Num("9".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("123456789"), &[Token::Num("123456789".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("0.0"), &[Token::Num("0.0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("0.5"), &[Token::Num("0.5".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("1.0"), &[Token::Num("1.0".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("1.25"), &[Token::Num("1.25".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("125.125"), &[Token::Num("125.125".to_string()), Token::Eof]);
    }

    #[test]
    fn test_lex_op_code() {
        assert_eq!(lex_to_tokens("!"), &[Token::OpCode("!".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("#"), &[Token::OpCode("#".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("$"), &[Token::OpCode("$".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("%"), &[Token::OpCode("%".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("&"), &[Token::OpCode("&".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("*"), &[Token::OpCode("*".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("+"), &[Token::OpCode("+".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("."), &[Token::OpCode(".".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("/"), &[Token::OpCode("/".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("<"), &[Token::OpCode("<".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("="), &[Token::OpCode("=".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens(">"), &[Token::OpCode(">".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("?"), &[Token::OpCode("?".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("@"), &[Token::OpCode("@".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("\\"), &[Token::OpCode("\\".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("^"), &[Token::OpCode("^".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("|"), &[Token::OpCode("|".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("-"), &[Token::OpCode("-".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens("~"), &[Token::OpCode("~".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens(":"), &[Token::OpCode(":".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens(":="), &[Token::ColonEq, Token::Eof]);
        assert_eq!(lex_to_tokens("!!"), &[Token::OpCode("!!".to_string()), Token::Eof]);
        assert_eq!(lex_to_tokens(">>="), &[Token::OpCode(">>=".to_string()), Token::Eof]);
    }

    #[test]
    fn test_lex_comma() {
        assert_eq!(lex_to_tokens(","), &[Token::Comma, Token::Eof]);
    }

    #[test]
    fn test_lex_paren() {
        assert_eq!(lex_to_tokens("{"), &[Token::LBrace, Token::Eof]);
        assert_eq!(lex_to_tokens("}"), &[Token::RBrace, Token::Eof]);
        assert_eq!(lex_to_tokens("("), &[Token::LParen, Token::Eof]);
        assert_eq!(lex_to_tokens(")"), &[Token::RParen, Token::Eof]);
    }

    #[test]
    fn test_lex_tokens() {
        assert_eq!(lex_to_tokens("a := { 1, 2 };"), &[
            Token::Ident("a".to_string()),
            Token::ColonEq,
            Token::LBrace,
            Token::Num("1".to_string()),
            Token::Comma,
            Token::Num("2".to_string()),
            Token::RBrace,
            Token::Semicolon,
            Token::Eof,
        ]);
    }

    #[test]
    fn test_lex_returns_infos() {
        let s = "a := { 1, 2 };";
        assert!(lex_to_infos(s).into_iter().zip(&[
            StrInfo::new(1, 1, &s[0..1], s),
            StrInfo::new(1, 3, &s[2..4], s),
            StrInfo::new(1, 6, &s[5..6], s),
            StrInfo::new(1, 8, &s[7..8], s),
            StrInfo::new(1, 9, &s[8..9], s),
            StrInfo::new(1, 11, &s[10..11], s),
            StrInfo::new(1, 13, &s[12..13], s),
            StrInfo::new(1, 14, &s[13..14], s),
            StrInfo::eof(),
        ]).all(|(actual, expected)| actual.strict_eq(expected)));
    }

    #[test]
    fn test_lex_failed() {
        assert_eq!(lex_to_err_msgs("あ"), &[
            "(1, 1): Invalid token found: `あ`",
        ]);
        assert_eq!(lex_to_err_msgs("abcあ\nい"), &[
            "(1, 4): Invalid token found: `あ`",
            "(2, 1): Invalid token found: `い`",
        ]);
    }
}
