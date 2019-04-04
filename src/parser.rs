use std::str::FromStr;

pub struct Parser {
    offset: usize,
    raw: Vec<u8>,
}

impl Parser {
    pub fn new(raw: Vec<u8>) -> Self {
        return Parser {
            raw: raw,
            offset: 0,
        };
    }

    fn check_capacity(&self, field_length: usize) {
        if self.offset + field_length > self.raw.len() {
            panic!("raw does not contains enough data")
        }
    }

    pub fn parse_string(&mut self, field_length: usize) -> String {
        self.check_capacity(field_length);

        let start = self.offset;
        let end = self.offset + field_length;

        // TODO  : this move should be don after the parsing of the slice
        // but we have this error : https://stackoverflow.com/questions/47618823/cannot-borrow-as-mutable-because-it-is-also-borrowed-as-immutable
        self.move_offset(field_length);

        let slice: &[u8] = &self.raw[start..end];

        std::str::from_utf8(slice).unwrap().trim().to_string()
    }

    pub fn move_offset(&mut self, field_length: usize) -> &mut Self {
        self.offset += field_length;
        self
    }

    pub fn parse_number<T: FromStr>(&mut self, field_length: usize) -> T
    where
        T: FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        self.check_capacity(field_length);

        let string = self.parse_string(field_length);
        string.parse::<T>().unwrap()
    }

    pub fn parse_string_list(&mut self, list_size: u64, field_length: usize) -> Vec<String> {
        (0..list_size as usize)
            .map(|_| self.parse_string(field_length))
            .collect()
    }

    pub fn parse_number_list<T: FromStr>(&mut self, list_size: u64, field_length: usize) -> Vec<T>
    where
        T: FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        self.parse_string_list(list_size, field_length)
            .into_iter()
            .map(|v| v.parse::<T>().unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn parse() {
        let mut parser = get_parser("12345678");
        assert_eq!("1234", parser.parse_string(4));
        assert_eq!(5678, parser.parse_number::<usize>(4));

        assert_eq!(8, parser.offset);
    }

    fn to_byte_array(string: &str) -> Vec<u8> {
        String::from(string).into_bytes()
    }

    fn get_parser(string: &str) -> Parser {
        // create a byte slice of the string
        Parser::new(to_byte_array(string))
    }

    #[test]
    #[should_panic]
    fn parse_with_no_enough_data() {
        let mut parser = get_parser("1234");
        parser.parse_string(5);
    }

    #[test]
    #[should_panic]
    fn parse_wrong_integer() {
        let mut parser = get_parser("hello");
        parser.parse_number::<usize>(4);
    }
}
