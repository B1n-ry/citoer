use std::{error::Error, future::Future, io, pin::Pin};

use chrono::{DateTime, Utc};

use crate::{media_adapters::slack::SlackAdapter, SaveData};

mod slack;

pub trait MediaAdapter {
    fn get_messages(
        &self,
        last_message_time: &Option<DateTime<Utc>>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SaveData>, Box<dyn Error>>> + Send>>;
}

pub async fn get_adapter(name: &str) -> Result<Box<dyn MediaAdapter>, io::Error> {
    match name {
        "slack" => SlackAdapter::new().map(|adapter| Box::new(adapter) as Box<dyn MediaAdapter>),
        _ => Err(io::Error::new(io::ErrorKind::NotFound, "No Adapter Found")),
    }
}
