use anyhow::{bail, Result};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Bencode {
    Dictionary(HashMap<String, Bencode>),
    List(Vec<Bencode>),
    Integer(isize),
    Bytes(Vec<u8>),
}

pub struct Parser {
    data: Vec<u8>,
    current: usize,
}

impl Parser {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Bencode> {
        match self.peek() {
            // dictionary
            b'd' => {
                self.advance();
                let mut dict = HashMap::new();
                while self.peek() != &b'e' {
                    let key = self.parse()?;
                    let value = self.parse()?;

                    if let Bencode::Bytes(key) = key {
                        let key = String::from_utf8(key)?;
                        dict.insert(key, value);
                    } else {
                        bail!("key is not a string! {:?}", key);
                    }
                }

                Ok(Bencode::Dictionary(dict))
            }

            // list
            b'l' => {
                self.advance();
                let mut list = vec![];
                while self.peek() != &b'e' {
                    let value = self.parse()?;
                    list.push(value);
                }
                Ok(Bencode::List(list))
            }

            // integer
            b'i' => {
                self.advance();
                let value = String::from_utf8(self.advance_to(b'e'))?.parse::<isize>()?;
                Ok(Bencode::Integer(value))
            }

            // bytes
            _x @ b'0'..=b'9' => {
                let size = String::from_utf8(self.advance_to(b':'))?.parse::<usize>()?;
                let content = self.advance_exact(size);

                Ok(Bencode::Bytes(content))
            }

            x => {
                panic!("Unknwon symbol {:?}", x)
            }
        }
    }

    fn advance_exact(&mut self, size: usize) -> Vec<u8> {
        let mut data = vec![];
        for _ in 0..size {
            data.push(self.advance());
        }
        data
    }

    /// advances up to the specified char and consumes it without returning it
    fn advance_to(&mut self, char: u8) -> Vec<u8> {
        let mut data = vec![];
        while self.peek() != &char {
            data.push(self.advance());
        }
        self.advance();
        data
    }

    fn advance(&mut self) -> u8 {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> u8 {
        self.data[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.current > self.data.len()
    }

    fn peek(&self) -> &u8 {
        &self.data[self.current]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_dict() -> Result<()> {
        let data = "d5:monthi4e4:name5:aprile".as_bytes();
        let parsed = Parser::new(data.to_vec()).parse()?;

        let mut expected = HashMap::new();
        expected.insert(String::from("month"), Bencode::Integer(4));
        expected.insert(
            String::from("name"),
            Bencode::Bytes("april".as_bytes().to_vec()),
        );
        let expected = Bencode::Dictionary(expected);
        assert!(parsed == expected);

        Ok(())
    }

    #[test]
    fn integer() -> Result<()> {
        let data = "i1234e".as_bytes();
        let parsed = Parser::new(data.to_vec()).parse()?;

        assert!(matches!(parsed, Bencode::Integer(1234)));
        Ok(())
    }

    #[test]
    fn list() -> Result<()> {
        let data = "li2e3:fooe".as_bytes();
        let parsed = Parser::new(data.to_vec()).parse()?;
        let expected = Bencode::List(vec![
            Bencode::Integer(2),
            Bencode::Bytes("foo".as_bytes().to_vec()),
        ]);

        assert!(parsed == expected);
        Ok(())
    }
}
