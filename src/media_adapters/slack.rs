use chrono::{DateTime, Utc};

use crate::{media_adapters::MediaAdapter, SaveData};
use std::{error::Error, future::Future, io, pin::Pin};

pub struct SlackAdapter {}

impl MediaAdapter for SlackAdapter {
    fn get_messages(
        &self,
        last_message_time: &Option<DateTime<Utc>>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<SaveData>, Box<dyn Error>>> + Send>> {
        Box::pin(async move { Ok(Vec::new()) })
    }
}

impl SlackAdapter {
    pub fn new() -> Result<Self, io::Error> {
        Ok(SlackAdapter {})
    }
}
