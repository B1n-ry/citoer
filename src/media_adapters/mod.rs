use std::{error::Error, io};

use chrono::{DateTime, Utc};

use crate::{media_adapters::slack::SlackAdapter, PinnedAsync, SaveData};

mod slack;

pub trait MediaAdapter {
    fn get_messages<'a>(
        &'a self,
        last_message_time: &'a Option<DateTime<Utc>>,
    ) -> PinnedAsync<'a, Result<Vec<SaveData>, Box<dyn Error>>>;
}

pub async fn get_adapter(name: &str) -> Result<Box<dyn MediaAdapter>, io::Error> {
    match name {
        "slack" => SlackAdapter::new().map(|adapter| Box::new(adapter) as Box<dyn MediaAdapter>),
        _ => Err(io::Error::new(io::ErrorKind::NotFound, "No Adapter Found")),
    }
}
