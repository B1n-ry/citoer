pub mod google_sheets;

use chrono::{DateTime, Utc};

use crate::storage_adapters::google_sheets::GoogleSheetsAdapter;
use crate::{PinnedAsync, SaveData};
use std::error::Error;
use std::io;

pub trait StorageAdapter {
    fn save<'a>(&'a self, data: &'a [SaveData]) -> PinnedAsync<'a, Result<(), Box<dyn Error>>>;

    fn get_most_recent_time<'a>(&'a self) -> PinnedAsync<'a, Option<DateTime<Utc>>>;
}

pub async fn get_adapter(name: &str) -> Result<Box<dyn StorageAdapter>, io::Error> {
    match name {
        "google_sheets" => GoogleSheetsAdapter::new()
            .await
            .map(|o| Box::new(o) as Box<dyn StorageAdapter>),
        _ => Err(io::Error::new(io::ErrorKind::NotFound, "No adapter found")),
    }
}
