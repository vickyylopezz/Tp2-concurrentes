use crate::{action::*, errors::Error, payment_method::Method};

const TYPE: usize = 0;
const CLIENT_ID: usize = 1;
const PRICE: usize = 2;
const METHOD: usize = 3;
const SHOP_ID_BLOCK: usize = 2;
const SHOP_ID_COMPLETE: usize = 4;
const SHOP_ID_FAIL: usize = 2;
pub struct MessageParser {}

impl MessageParser {
    pub fn parse(s: String) -> Result<Action, Error> {
        let words: Vec<&str> = s.split(' ').collect();
        match words[TYPE] {
            "block" => MessageParser::parse_block(words),
            "complete" => MessageParser::parse_completion(words),
            "ACK" => MessageParser::parser_ack(words),
            "notEnough" => MessageParser::parser_not_enough(words),
            "alreadyBlocked" => MessageParser::parser_already_blocked(words),
            "fail" => MessageParser::parse_failure(words),
            "TRY" => MessageParser::parser_try(words),
            "DOWN" => MessageParser::parser_down(words),
            "UP" => MessageParser::parser_up(words),
            "SYNC" => MessageParser::parser_sync(words),
            "SYNCSTART" => MessageParser::parse_sync_start(words),
            "SYNCEND" => MessageParser::parse_sync_end(words),
            _ => Err(Error::InvalidMessageFormat),
        }
    }

    fn parse_sync_start(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 1 {
            return Err(Error::InvalidMessageFormat);
        }
        Ok(Action::SyncStart)
    }

    fn parse_sync_end(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 1 {
            return Err(Error::InvalidMessageFormat);
        }
        Ok(Action::SyncEnd)
    }

    fn parser_sync(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 2 {
            return Err(Error::InvalidMessageFormat);
        }
        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::Sync(client_id))
    }

    fn parser_ack(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 1 {
            return Err(Error::InvalidMessageFormat);
        }
        Ok(Action::Ack)
    }

    fn parser_down(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 1 {
            return Err(Error::InvalidMessageFormat);
        }
        Ok(Action::Down)
    }
    fn parser_up(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 1 {
            return Err(Error::InvalidMessageFormat);
        }
        Ok(Action::Up)
    }

    fn parser_try(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 1 {
            return Err(Error::InvalidMessageFormat);
        }
        Ok(Action::Try)
    }
    fn parse_block(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 3 {
            return Err(Error::InvalidMessageFormat);
        }
        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        let s: &str = words[SHOP_ID_BLOCK];
        let shop_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };

        Ok(Action::Block(client_id, shop_id))
    }

    fn parse_completion(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 5 {
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
        let shop_id: u32 = match words[SHOP_ID_COMPLETE].parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::CompleteOrder(client_id, price, method, shop_id))
    }

    fn parser_not_enough(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 2 {
            return Err(Error::InvalidMessageFormat);
        }
        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::NotEnoughPoints(client_id))
    }

    fn parser_already_blocked(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 2 {
            return Err(Error::InvalidMessageFormat);
        }
        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::ClientAlreadyBlocked(client_id))
    }

    fn parse_failure(words: Vec<&str>) -> Result<Action, Error> {
        if words.len() != 3 {
            return Err(Error::InvalidMessageFormat);
        }

        let s: &str = words[CLIENT_ID];
        let client_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };

        let s: &str = words[SHOP_ID_FAIL];
        let shop_id: u32 = match s.parse::<u32>() {
            Ok(i) => i,
            Err(_) => return Err(Error::InvalidMessageFormat),
        };
        Ok(Action::FailOrder(client_id, shop_id))
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
        let s: String = "block 123 0".to_string();
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
        let s: String = "complete 123 10 cash 0".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic_on_non_numeric_price() {
        let s: String = "complete 123 dolares cash 0".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_complete_points() {
        let s: String = "complete 123 10 points 0".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    #[should_panic]
    fn panic_on_invalid_method() {
        let s: String = "complete 123 10 credit 0".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_fail() {
        let s: String = "fail 123 0".to_string();
        MessageParser::parse(s).unwrap();
    }

    #[test]
    fn can_parse_already_blocked() {
        let s: String = "alreadyBlocked 123".to_string();
        MessageParser::parse(s).unwrap();
    }
}
