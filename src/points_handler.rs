use std::collections::HashMap;

use crate::errors::Error;

#[derive(Clone)]
pub struct PointsHandler {
    pub points: HashMap<u32, (i32, bool)>,
}

impl PointsHandler {
    /// Creates a new instance of [`PointsHandler`].
    pub fn new() -> PointsHandler {
        PointsHandler {
            points: HashMap::new(),
        }
    }

    /// Returns the current information associated with the client id.
    /// If the client account does not exist, it is created.
    fn get_client(&mut self, client_id: u32) -> (i32, bool) {
        match self.points.get(&client_id) {
            Some(info) => *info,
            None => {
                let init_value = 0;
                self.points.insert(client_id, (0, false));
                (init_value, false)
            }
        }
    }

    /// Blocks the client.
    /// Returns error If the client was already blocked.
    pub fn block(&mut self, client_id: u32) -> Result<(), Error> {
        let current = self.clone().get_client(client_id);
        if !current.1 {
            self.points.insert(client_id, (current.0, true));
        } else {
            return Err(Error::UserAlreadyBlocked);
        }

        Ok(())
    }

    /// Blocks the client.
    /// Returns error If the client was already blocked.
    pub fn unblock(&mut self, client_id: u32) {
        let current = self.clone().get_client(client_id);
        self.points.insert(client_id, (current.0, false));
    }

    /// Updates the points associated with the client id.
    /// Returns error If there are no enough points to subtract in the client account.
    pub fn update_points(&mut self, client_id: u32, points: i32) -> Result<(), Error> {
        let current = self.clone().get_client(client_id);
        let updated_points = current.0 + points;
        if updated_points >= 0 {
            self.points.insert(client_id, (updated_points, current.1));
        } else {
            return Err(Error::NotEnoughPoints);
        }
        Ok(())
    }

    /// Acumulates points to the associated client id.
    pub fn acumulate(&mut self, client_id: u32, points: i32) {
        if self.update_points(client_id, points).is_ok() {
            let current = self.clone().get_client(client_id);
            let updated_points = current.0 + points;
            self.points.insert(client_id, (updated_points, current.1));
        }
    }
}

impl Default for PointsHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::Error;

    use super::PointsHandler;

    #[test]
    pub fn test_01_add_points_to_new_client() {
        let mut client_points = PointsHandler::new();

        let got = client_points.get_client(0);
        assert_eq!(got.0, 0);
    }

    #[test]
    pub fn test_02_add_points_to_existent_client() {
        let mut client_points = PointsHandler::new();

        client_points
            .update_points(0, 10)
            .expect("Error when updating points");
        let got = client_points.get_client(0);
        assert_eq!(got.0, 10);
    }

    #[test]
    pub fn test_03_subtract_points_to_client_with_enough_points() {
        let mut client_points = PointsHandler::new();

        client_points
            .update_points(0, 10)
            .expect("Error when adding points");
        client_points
            .update_points(0, -5)
            .expect("Error when subtracting points");
        let got = client_points.get_client(0);

        assert_eq!(got.0, 5);
    }

    #[test]
    pub fn test_04_subtract_points_to_client_with_not_enough_points() {
        let mut client_points = PointsHandler::new();

        client_points
            .update_points(0, 10)
            .expect("Error when adding points");
        let err_got = client_points
            .update_points(0, -15)
            .expect_err("Error when subtracting points");

        assert_eq!(err_got, Error::NotEnoughPoints);
    }
}
