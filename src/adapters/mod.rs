use crate::adapters::google_sheets::GoogleSheetsAdapter;

pub mod google_sheets;

pub trait StorageAdapter {
    fn save(&self, data: &GroupedData) -> Result<(), String>;
}

pub fn get_adapter(name: &str) -> Option<Box<dyn StorageAdapter>> {
    match name {
        "google_sheets" => Some(Box::new(GoogleSheetsAdapter::new())),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct GroupedData<'a> {
    pub quote: &'a str,
    pub quoter: &'a str,
    pub quotee: &'a str,
    pub receiver: Option<&'a str>,
}
