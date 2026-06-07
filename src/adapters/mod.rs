use crate::adapters::google_sheets::GoogleSheetsAdapter;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub mod google_sheets;

pub trait StorageAdapter {
    fn save<'a>(
        &'a self,
        data: &'a GroupedData<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;
}

pub async fn get_adapter(name: &str) -> Result<Box<dyn StorageAdapter>, Box<dyn Error>> {
    match name {
        "google_sheets" => GoogleSheetsAdapter::new()
            .await
            .map(|o| Box::new(o) as Box<dyn StorageAdapter>),
        _ => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No adapter found",
        ))),
    }
}

#[derive(Debug, Clone)]
pub struct GroupedData<'a> {
    pub quote: &'a str,
    pub quoter: &'a str,
    pub quotee: &'a str,
    pub receiver: Option<&'a str>,
}
