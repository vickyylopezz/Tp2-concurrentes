use std::collections::HashMap;

use crate::errors::Error;

#[derive(Clone)]
pub struct PointsHandler {
    pub points: HashMap<u32, i32>,
}

impl PointsHandler {
    /// Creates a new instance of [`PointsHandler`].
    pub fn new() -> PointsHandler {
        PointsHandler {
            points: HashMap::new(),
        }
    }

    /// Returns the current points associated with the client id.
    /// If the client account does not exist, it is created.
    fn get_points(&mut self, client_id: u32) -> i32 {
        match self.points.get(&client_id) {
            Some(points) => *points,
            None => {
                let init_value = 0;
                self.points.insert(client_id, 0);
                init_value
            }
        }
    }

    /// Updates the points associated with the client id.
    /// Returns error If there are no enough points to subtract in the client account.
    pub fn update_points(&mut self, client_id: u32, points: i32) -> Result<(), Error> {
        let current = self.clone().get_points(client_id);
        let updated_points = current + points;
        if updated_points >= 0 {
            self.points.insert(client_id, updated_points);
        } else {
            return Err(Error::NotEnoughPoints);
        }

        Ok(())
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

        let got = client_points.get_points(0);
        assert_eq!(got, 0);
    }

    #[test]
    pub fn test_02_add_points_to_existent_client() {
        let mut client_points = PointsHandler::new();

        client_points
            .update_points(0, 10)
            .expect("Error when updating points");
        let got = client_points.get_points(0);
        assert_eq!(got, 10);
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
        let got = client_points.get_points(0);

        assert_eq!(got, 5);
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
