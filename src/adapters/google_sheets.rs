use super::{GroupedData, StorageAdapter};

pub struct GoogleSheetsAdapter {}

impl GoogleSheetsAdapter {
    pub fn new() -> Self {
        Self {}
    }
}

impl StorageAdapter for GoogleSheetsAdapter {
    fn save(&self, data: &GroupedData) -> Result<(), String> {
        Ok(())
    }
}
