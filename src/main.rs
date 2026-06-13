mod media_adapters;
mod storage_adapters;

use chrono::{DateTime, Utc};
use dotenv;
use regex::Regex;
use std::{env, io};

use crate::MediaMessage::Full;

const QUOTE_REGEX: &str = r#"(?P<quote1>["\"”])?(?P<text>.*?)(?P<quote2>["\"”])?\s*-\s*@?(?P<quotee>.*?)(?P<till>\s+till\s+@(?P<receiver>.*))?$"#;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().ok();

    let quote_regex = env::var("QUOTE_REGEX").unwrap_or(String::from(QUOTE_REGEX));

    let media_adapter_name = env::var("MEDIA_ADAPTER").unwrap_or(String::from("slack"));
    let storage_adapter_name = env::var("STORAGE_ADAPTER").unwrap_or(String::from("google_sheets"));

    let media_adapter = media_adapters::get_adapter(&media_adapter_name)
        .await
        .expect(&format!(
            "No media adapter found for '{}'",
            media_adapter_name
        ));
    let storage_adapter = storage_adapters::get_adapter(&storage_adapter_name)
        .await
        .expect(&format!(
            "No storage adapter found for '{}'",
            storage_adapter_name
        ));

    let latest_saved_time = storage_adapter.get_most_recent_time().await;

    let messages = media_adapter
        .get_messages(&latest_saved_time)
        .await
        .expect("Failed to fetch messages");

    let grouped: Vec<SaveData> = messages
        .iter()
        .map(|m| {
            let grouped_message = match m.message.group(&quote_regex) {
                Ok(ok) => ok,
                Err(e) => panic!("{}", e),
            };
            SaveData {
                message: grouped_message,
                ..m.clone()
            }
        })
        .collect();

    storage_adapter.save(&grouped);
}

#[derive(Clone)]
struct SaveData {
    id: String,
    message: MediaMessage,
    author: String,
    time: Option<DateTime<Utc>>,
}
#[derive(Clone)]
enum MediaMessage {
    Full {
        message: String,
    },
    Grouped {
        quote: String,
        quotee: String,
        receiver: Option<String>,
    },
}

impl MediaMessage {
    fn group(&self, regex: &str) -> Result<Self, io::Error> {
        let regex = Regex::new(regex).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                match e {
                    regex::Error::Syntax(err) => format!("Regex is not valid: {}", err),
                    regex::Error::CompiledTooBig(err) => {
                        format!("Regex is too big. Exceeded size limit {}", err)
                    }
                    _ => String::from("Unknown error generating regex"),
                },
            )
        })?;

        let Full { message } = self else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Already ungrouped",
            ));
        };

        let caps = regex.captures(&message).ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "Data does not match regex; no match found",
        ))?;

        let Some((quote, quotee)) = caps.name("text").zip(caps.name("quotee")) else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Missing required regex captures",
            ));
        };

        let groups = MediaMessage::Grouped {
            quote: String::from(quote.as_str()),
            quotee: String::from(quotee.as_str()),
            receiver: caps.name("receiver").map(|m| String::from(m.as_str())),
        };

        Ok(groups)
    }
}
