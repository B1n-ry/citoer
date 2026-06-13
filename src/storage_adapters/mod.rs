pub mod google_sheets;

use chrono::{DateTime, Utc};

use crate::storage_adapters::google_sheets::GoogleSheetsAdapter;
use crate::SaveData;
use std::error::Error;
use std::future::Future;
use std::io;
use std::pin::Pin;

pub trait StorageAdapter {
    fn save<'a>(
        &'a self,
        data: &'a Vec<SaveData>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>>;

    fn get_most_recent_time<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Option<DateTime<Utc>>> + Send + 'a>>;
}

pub async fn get_adapter(name: &str) -> Result<Box<dyn StorageAdapter>, io::Error> {
    match name {
        "google_sheets" => GoogleSheetsAdapter::new()
            .await
            .map(|o| Box::new(o) as Box<dyn StorageAdapter>),
        _ => Err(io::Error::new(io::ErrorKind::NotFound, "No adapter found")),
    }
}
