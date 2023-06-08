use crate::{action::*, errors::Error, payment_method::Method};

const TYPE: usize = 0;
const CLIENT_ID: usize = 1;
const PRICE: usize = 2;
const METHOD: usize = 3;

pub struct MessageParser {}

impl MessageParser {
    pub fn parse(s: String) -> Result<Action, Error> {
        let words: Vec<&str> = s.split(' ').collect();
        match words[TYPE] {
            "block" => MessageParser::parse_block(words),
            "complete" => MessageParser::parse_completion(words),
            "fail" => MessageParser::parse_failure(words),
            _ => Err(Error::InvalidMessageFormat),
        }
    }

    fn parse_block(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 2 {
            return Err(Error::InvalidMessageFormat);
        }
        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::Block(client_id))
    }

    fn parse_completion(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 4 {
            return Err(Error::InvalidMessageFormat);
        }

        let client_id: u32 = match words[CLIENT_ID].parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        let price: u32 = match words[PRICE].parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        let method: Method = match words[METHOD] {
            "cash" => Method::Cash,
            "points" => Method::Points,
            _ => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::CompleteOrder(client_id, price, method))
    }

    fn parse_failure(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 2 {
            return Err(Error::InvalidMessageFormat);
        }

        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::FailOrder(client_id))
    }
}

#[cfg(test)]
mod message_parser_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn panic_on_wrong_message() {
        let s: String = "invalid".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_block() {
        let s: String = "block 123".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic_on_non_numeric_client_id() {
        let s: String = "block persona".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_complete_cash() {
        let s: String = "complete 123 10 cash".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic_on_non_numeric_price() {
        let s: String = "complete 123 dolares cash".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_complete_points() {
        let s: String = "complete 123 10 points".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic_on_invalid_method() {
        let s: String = "complete 123 10 credit".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_fail() {
        let s: String = "fail 123".to_string();
        MessageParser::parse(s).unwrap();
    }
}
