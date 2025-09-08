use std::slice::SliceIndex;

#[derive(Clone, Debug)]
pub struct Tokenizer<'a> {
    s: &'a str,
    index: usize,
    errors: Vec<ParseError>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            s,
            index: 0,
            errors: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.s.len() == self.index
    }

    fn look(&self) -> Option<char> {
        self.s[self.index..].chars().next()
    }

    fn look_at(&self, idx: usize) -> Option<char> {
        self.s.get(self.index + idx..)?.chars().next()
    }

    fn read(&mut self) -> Option<char> {
        let c = self.look()?;
        self.index += c.len_utf8();
        Some(c)
    }

    fn unread(&mut self, c: Option<char>) {
        if let Some(c) = c {
            self.index -= c.len_utf8();
        }
    }

    fn error(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    fn cursor(&self) -> usize {
        self.index
    }

    fn get<I: SliceIndex<str>>(&self, index: I) -> Option<&<I as SliceIndex<str>>::Output> {
        self.s.get(index)
    }

    fn as_str(&self) -> &'a str {
        &self.s[self.index..]
    }

    fn is_ident_start_code_point(c: char) -> bool {
        c.is_ascii_alphabetic() || !c.is_ascii() || c == '_'
    }

    fn next_is_ident_start_code_point(&self) -> bool {
        if let Some(c) = self.look() {
            Self::is_ident_start_code_point(c)
        } else {
            false
        }
    }

    fn is_ident_code_point(c: char) -> bool {
        Self::is_ident_start_code_point(c) || c.is_ascii_digit() || c == '-'
    }

    fn next_is_ident_code_point(&self) -> bool {
        if let Some(c) = self.look() {
            Self::is_ident_code_point(c)
        } else {
            false
        }
    }

    fn are_valid_escape(s: &str) -> bool {
        let mut chars = s.chars();
        if let Some(first) = chars.next()
            && let Some(second) = chars.next()
        {
            first == '\\' && second != '\n'
        } else {
            false
        }
    }

    fn next_two_are_valid_escape(&self) -> bool {
        Self::are_valid_escape(self.as_str())
    }

    fn next_three_would_start_an_ident_sequence(&self) -> bool {
        let mut chars = self.as_str().chars();

        match chars.next() {
            Some('-') => {
                if let Some(second) = chars.next() {
                    Self::is_ident_start_code_point(second)
                        || second == '-'
                        || Self::are_valid_escape(&self.as_str()[1..])
                } else {
                    false
                }
            }
            Some(c) if Self::is_ident_start_code_point(c) => true,
            Some('\\') => self.next_two_are_valid_escape(),
            _ => false,
        }
    }

    fn read_an_escape_code_point(&mut self) -> char {
        const MAXIMUM_ALLOWED_CODE_POINT: u32 = 0x10ffff;

        self.read();

        match self.read() {
            Some(c) if c.is_ascii_hexdigit() => {
                let start = self.cursor() - c.len_utf8();
                for _ in 0..5 {
                    if let Some(c) = self.read() {
                        if !c.is_ascii_hexdigit() {
                            self.unread(Some(c));
                            break;
                        }
                    }
                }
                let end = self.cursor();
                let s = self.get(start..end).unwrap();
                let value = u32::from_str_radix(s, 16).unwrap();
                let character = if let Some(c) = char::from_u32(value)
                    && value != 0
                    && value <= MAXIMUM_ALLOWED_CODE_POINT
                {
                    c
                } else {
                    '\u{fffd}'
                };
                self.skip_whitespace();
                character
            }
            None => {
                self.error(ParseError::UnexpectedNullCharacter);
                return '\u{fffd}';
            }
            Some(c) => c,
        }
    }

    fn seek(&mut self, len: usize) {
        self.index += len;
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.read() {
            if !c.is_whitespace() {
                self.unread(Some(c));
                break;
            }
        }
    }

    fn read_an_ident_sequence(&mut self) -> String {
        let mut ident = String::new();

        loop {
            match self.look() {
                Some(c) if Self::is_ident_code_point(c) => {
                    self.read();
                    ident.push(c)
                }
                Some('\\') if self.next_two_are_valid_escape() => {
                    let c = self.read_an_escape_code_point();
                    ident.push(c);
                }
                _ => break,
            }
        }

        ident
    }

    fn is_number(s: &str) -> bool {
        let mut chars = s.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        match first {
            '+' | '-' => {
                if let Some(second) = chars.next() {
                    if second.is_ascii_digit() {
                        true
                    } else if let Some(third) = chars.next()
                        && second == '.'
                    {
                        third.is_ascii_digit()
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            '.' => {
                matches!(chars.next(), Some(second) if second.is_ascii_digit())
            }
            c if c.is_ascii_digit() => true,
            _ => false,
        }
    }

    fn next_is_number(&self) -> bool {
        Self::is_number(self.as_str())
    }

    fn read_number_len_and_type(&mut self) -> (usize, bool) {
        let mut len = 0;
        let mut is_integer = true;

        if let Some(c) = self.look()
            && (c == '+' || c == '-')
        {
            len += c.len_utf8();
            self.read();
        }
        while let Some(c) = self.look()
            && c.is_ascii_digit()
        {
            self.read();
            len += c.len_utf8();
        }
        if let Some(c) = self.look()
            && c == '.'
        {
            self.read();
            len += c.len_utf8();
            is_integer = false;

            while let Some(c) = self.look()
                && c.is_ascii_digit()
            {
                self.read();
                len += c.len_utf8();
            }
        }
        if let Some(c) = self.look()
            && (c == 'E' || c == 'e')
            && let Some(c2) = self.look_at(c.len_utf8())
            && c2.is_ascii_digit()
        {
            self.read();
            len += c.len_utf8();
            is_integer = false;

            while let Some(c) = self.look()
                && (c.is_ascii_digit() || c == '+' || c == '-')
            {
                self.read();
                len += c.len_utf8();
            }
        }

        (len, is_integer)
    }

    fn read_number(&mut self) -> Num {
        let cursor = self.cursor();
        let (len, is_integer) = self.read_number_len_and_type();
        let repr = self.get(cursor..cursor + len).unwrap();
        if is_integer {
            Num::Integer(repr.parse::<i64>().or::<()>(Ok(0)).unwrap())
        } else {
            Num::Floating(repr.parse::<f64>().or::<()>(Ok(0.0)).unwrap())
        }
    }

    fn read_numeric_token(&mut self) -> Token {
        let value = self.read_number();
        if self.next_three_would_start_an_ident_sequence() {
            let unit = self.read_an_ident_sequence();
            Token::Dimension { value, unit }
        } else if self.as_str().starts_with('%') {
            self.read();
            Token::Percentage(value)
        } else {
            Token::Number(value)
        }
    }

    fn read_remnants_of_a_bad_url(&mut self) {
        loop {
            if [Some(')'), None].contains(&self.look()) {
                break;
            } else if self.next_two_are_valid_escape() {
                self.read_an_escape_code_point();
            } else {
                self.read();
            }
        }
    }

    fn read_a_url_token(&mut self) -> Token {
        let mut value = String::new();
        self.skip_whitespace();
        loop {
            match self.read() {
                Some(')') => {
                    self.unread(Some(')'));
                    break Token::Url(value);
                }
                None => {
                    self.error(ParseError::UnexpectedEof);
                    break Token::Url(value);
                }
                Some(c) if c.is_whitespace() => {
                    self.skip_whitespace();
                    if [None, Some(')')].contains(&self.look()) {
                        break Token::Url(value);
                    } else {
                        self.read_remnants_of_a_bad_url();
                        break Token::BadUrl;
                    }
                }
                Some(c) if ['"', '\'', '('].contains(&c) => {
                    self.unread(Some(c));
                    break Token::Url(value);
                }
                Some('\\') => {
                    if self.next_two_are_valid_escape() {
                        let c = self.read_an_escape_code_point();
                        value.push(c);
                    } else {
                        self.error(ParseError::InvalidEscapeSequence);
                        break Token::BadUrl;
                    }
                }
                Some(c) => {
                    value.push(c);
                }
            }
        }
    }

    fn read_ident_like_token(&mut self) -> Token {
        let ident = self.read_an_ident_sequence();
        if ident.eq_ignore_ascii_case("url") && self.look() == Some('(') {
            self.read();
            self.skip_whitespace();
            let mut token;
            if let Some(c) = self.look()
                && (c == '"' || c == '\'')
            {
                self.read();
                token = self.read_a_url_token();
                if self.look() == Some(c) {
                    self.read();
                } else {
                    self.read_remnants_of_a_bad_url();
                    self.error(ParseError::ExpectedStringEnd);
                    token = Token::BadUrl;
                }
            } else {
                token = self.read_a_url_token();
            }
            self.skip_whitespace();
            if self.look() == Some(')') {
                self.read();
            } else {
                self.error(ParseError::ExpectedRParen);
                token = Token::BadUrl;
            }
            token
        } else if let Some(c) = self.look()
            && c == '('
        {
            self.read();
            Token::Function(ident)
        } else {
            Token::Ident(ident)
        }
    }

    fn skip_comment(&mut self) {
        if self.as_str().starts_with("/*") {
            self.read();
            self.read();

            loop {
                if self.is_empty() {
                    self.error(ParseError::UnexpectedEof);
                    break;
                }
                if self.as_str().starts_with("*/") {
                    self.read();
                    self.read();
                    break;
                }
                self.read();
            }
        }
    }

    pub fn step(&mut self) -> Option<Token> {
        self.skip_comment();

        let Some(c) = self.look() else {
            return None;
        };

        let token = match c {
            c if c.is_ascii_whitespace() => self.step_whitespace(),
            '"' => self.step_string(),
            '#' => self.step_hash(),
            '\'' => self.step_string(),
            '(' => self.step_lparen(),
            ')' => self.step_rparen(),
            '+' => self.step_plus(),
            ',' => self.step_comma(),
            '-' => self.step_hyphen_minus(),
            '.' => self.step_full_stop(),
            ':' => self.step_colon(),
            ';' => self.step_semicolon(),
            '<' => self.step_less_than_sign(),
            '@' => self.step_commercial_at(),
            '[' => self.step_lsquare(),
            '\\' => self.step_reverse_solidus(),
            ']' => self.step_rsquare(),
            '{' => self.step_lcurly(),
            '}' => self.step_rcurly(),
            c if c.is_ascii_digit() => self.step_digit(),
            c if Self::is_ident_start_code_point(c) => self.step_ident(),
            _ => self.step_anything_else(),
        };

        Some(token)
    }

    fn step_ident(&mut self) -> Token {
        self.read_ident_like_token()
    }

    fn step_anything_else(&mut self) -> Token {
        let c = self.read().unwrap();
        Token::Delim(c)
    }

    fn step_digit(&mut self) -> Token {
        self.read_numeric_token()
    }

    fn step_rcurly(&mut self) -> Token {
        self.read();
        Token::RCurly
    }

    fn step_lcurly(&mut self) -> Token {
        self.read();
        Token::LCurly
    }

    fn step_rsquare(&mut self) -> Token {
        self.read();
        Token::RSquare
    }

    fn step_reverse_solidus(&mut self) -> Token {
        if self.next_three_would_start_an_ident_sequence() {
            self.read_ident_like_token()
        } else {
            Token::Delim(self.read().unwrap())
        }
    }

    fn step_lsquare(&mut self) -> Token {
        self.read();
        Token::LSquare
    }

    fn step_commercial_at(&mut self) -> Token {
        let c = self.read().unwrap();
        if self.next_three_would_start_an_ident_sequence() {
            Token::AtKeyword(self.read_an_ident_sequence())
        } else {
            Token::Delim(c)
        }
    }

    fn step_less_than_sign(&mut self) -> Token {
        let c = self.read().unwrap();
        if self.as_str().starts_with("!--") {
            Token::Cdo
        } else {
            Token::Delim(c)
        }
    }

    fn step_semicolon(&mut self) -> Token {
        self.read();
        Token::Semicolon
    }

    fn step_colon(&mut self) -> Token {
        self.read();
        Token::Colon
    }

    fn step_full_stop(&mut self) -> Token {
        if self.next_is_number() {
            self.read_numeric_token()
        } else {
            Token::Delim(self.read().unwrap())
        }
    }

    fn step_hyphen_minus(&mut self) -> Token {
        if self.next_is_number() {
            self.read_numeric_token()
        } else if self.as_str().starts_with("-->") {
            for _ in 0..3 {
                self.read();
            }
            Token::Cdc
        } else if self.next_three_would_start_an_ident_sequence() {
            self.read_ident_like_token()
        } else {
            Token::Delim(self.read().unwrap())
        }
    }

    fn step_comma(&mut self) -> Token {
        self.read();
        Token::Comma
    }

    fn step_plus(&mut self) -> Token {
        let c = self.read().unwrap();
        if self.next_is_number() {
            self.read_numeric_token()
        } else {
            Token::Delim(c)
        }
    }

    fn step_lparen(&mut self) -> Token {
        self.read();
        Token::LParen
    }

    fn step_rparen(&mut self) -> Token {
        self.read();
        Token::RParen
    }

    fn step_hash(&mut self) -> Token {
        self.read();
        if self.next_is_ident_code_point() || self.next_two_are_valid_escape() {
            let mut type_flag = "";
            if self.next_three_would_start_an_ident_sequence() {
                type_flag = "id";
            }
            let value = self.read_an_ident_sequence();
            Token::Hash { value, type_flag }
        } else {
            Token::Delim('#')
        }
    }

    fn step_string(&mut self) -> Token {
        let mut string = String::new();
        let ending_code_point = self.read().unwrap();

        while let Some(c) = self.read() {
            match c {
                c if c == ending_code_point => {
                    return Token::String(string);
                }
                '\n' => {
                    self.error(ParseError::NewlineInString);
                    return Token::BadString;
                }
                '\\' => {
                    if let Some(c) = self.read()
                        && c != '\n'
                    {
                        let c = self.read_an_escape_code_point();
                        string.push(c);
                    }
                }
                _ => {
                    string.push(c);
                }
            }
        }
        self.error(ParseError::UnexpectedNullCharacter);
        Token::String(string)
    }

    fn step_whitespace(&mut self) -> Token {
        self.skip_whitespace();
        Token::Whitespace
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Function(String),
    AtKeyword(String),
    Hash {
        value: String,
        type_flag: &'static str,
    },
    String(String),
    BadString,
    Url(String),
    BadUrl,
    Delim(char),
    Number(Num),
    Percentage(Num),
    Dimension {
        value: Num,
        unit: String,
    },
    Whitespace,
    Cdo,
    Cdc,
    Colon,
    Semicolon,
    Comma,
    LSquare,
    RSquare,
    LParen,
    RParen,
    LCurly,
    RCurly,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParseError {
    UnexpectedNullCharacter,
    UnexpectedEof,
    UnexpectedCharacter,
    NewlineInString,
    InvalidEscapeSequence,
    ExpectedStringEnd,
    ExpectedRParen,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Num {
    Integer(i64),
    Floating(f64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_1() {
        let s = r#"
width: calc(100% - var(--gap));
height: calc(50px + 2 * var(--padding));
        "#;
        let mut tokenizer = Tokenizer::new(s);
        let mut log = String::new();
        while let Some(t) = tokenizer.step() {
            log.push_str(&format!("{:?}\n", t));
        }
        assert_eq!(
            &log,
            r#"Whitespace
Ident("width")
Colon
Whitespace
Function("calc")
Percentage(Integer(100))
Whitespace
Delim('-')
Whitespace
Function("var")
Ident("--gap")
RParen
RParen
Semicolon
Whitespace
Ident("height")
Colon
Whitespace
Function("calc")
Dimension { value: Integer(50), unit: "px" }
Whitespace
Delim('+')
Whitespace
Number(Integer(2))
Whitespace
Delim('*')
Whitespace
Function("var")
Ident("--padding")
RParen
RParen
Semicolon
Whitespace
"#
        );
    }

    #[test]
    fn test_tokenizer_2() {
        let s = r#"
@import url(https://example.com/style.css);

@import url("https://example.com/quoted.css");
@import url('https://example.com/quoted-single.css');

@import url(   https://example.com/space.css   );

@import url("https://example.com/bad.css');

@import url();

@import url(https://example.com/comment.css); /* コメント */
        "#;
        let mut tokenizer = Tokenizer::new(s);
        let mut log = String::new();
        while let Some(t) = tokenizer.step() {
            log.push_str(&format!("{:?}\n", t));
        }
        assert_eq!(
            &log,
            r#"Whitespace
AtKeyword("import")
Whitespace
Url("https://example.com/style.css")
Semicolon
Whitespace
AtKeyword("import")
Whitespace
Url("https://example.com/quoted.css")
Semicolon
Whitespace
AtKeyword("import")
Whitespace
Url("https://example.com/quoted-single.css")
Semicolon
Whitespace
AtKeyword("import")
Whitespace
Url("https://example.com/space.css")
Semicolon
Whitespace
AtKeyword("import")
Whitespace
BadUrl
Semicolon
Whitespace
AtKeyword("import")
Whitespace
Url("")
Semicolon
Whitespace
AtKeyword("import")
Whitespace
Url("https://example.com/comment.css")
Semicolon
Whitespace
Whitespace
"#
        );
    }
}
