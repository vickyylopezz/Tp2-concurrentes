use std::path::Path;

use crate::{errors::Error, orders::Order};

#[derive(Clone, Debug)]
pub struct InputController {
    pub filename: String,
}

impl InputController {
    /// Creates an [`InputController`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the user does not enter a filename.
    pub fn new(input: Option<String>) -> Result<InputController, Error> {
        let file = match input {
            Some(file) => file,
            None => return Err(Error::NotFileInput),
        };

        Ok(InputController { filename: file })
    }

    /// Converts the orders from a json file to a vector of orders if it can,
    /// returns an error if not.
    pub fn deserialize(self, orders: &str) -> Result<Vec<Order>, Error> {
        let result = match serde_json::from_str::<Vec<Order>>(orders) {
            Ok(orders) => orders,
            Err(_) => return Err(Error::WrongFileFormat),
        };

        Ok(result)
    }

    /// Reads the filename entered from user and returns a vector of orders if it can,
    /// returns an error if not.
    pub fn get_orders(self) -> Result<Vec<Order>, Error> {
        let dir = Path::new("resources/");
        let file = Path::new(&self.filename);
        let path = dir.join(file);

        let orders = match std::fs::read_to_string(path) {
            Ok(orders) => orders,
            Err(_e) => return Err(Error::FileNotFound),
        };

        Ok(self.deserialize(&orders))?
    }
}

#[cfg(test)]
mod tests {
    use crate::{errors::Error, input_controller::InputController};

    #[test]
    fn test01_get_a_valid_filename() {
        let controller =
            InputController::new(Some("orders.json".to_string())).expect("The filename is invalid");
        let expected_file = "orders.json".to_string();
        let got_file = controller.filename;
        assert_eq!(expected_file, got_file);
    }

    #[test]
    fn test02_not_get_a_filename() {
        let result =
            InputController::new(None).expect_err("You must enter a filename of the orders file");
        let err_expected = Error::NotFileInput;

        assert_eq!(result, err_expected);
    }

    #[test]
    fn test03_get_a_not_found_filename() {
        let controller = InputController::new(Some("pedidos.json".to_string()))
            .expect("The filename is invalid");
        let result = controller
            .get_orders()
            .expect_err("The filename was not found");
        let err_expected = Error::FileNotFound;

        assert_eq!(result, err_expected);
    }

    #[test]
    fn test04_get_an_order_without_all_fields() {
        let controller =
            InputController::new(Some("orders.json".to_string())).expect("The filename is invalid");
        let orders = "{\r\n    \"all\":[\r\n        {\r\n            \"water\": 10,\r\n            \"cocoa\": 2,\r\n            \"foam\": 2\r\n        }\r\n    ]\r\n}".to_string();
        let result = controller
            .deserialize(&orders)
            .expect_err("The order doesnt have all the ingredients");
        let err_expected = Error::WrongFileFormat;

        assert_eq!(result, err_expected);
    }
}
