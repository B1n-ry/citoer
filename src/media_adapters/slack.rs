use chrono::{DateTime, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::{media_adapters::MediaAdapter, MediaMessage, PinnedAsync, SaveData};
use std::{collections::HashMap, env, error::Error, io};

// Slack API types
#[derive(Debug, Deserialize)]
struct SlackHistoryResponse {
    ok: bool,
    messages: Option<Vec<SlackMessage>>,
    error: Option<String>,
}
#[derive(Debug, Deserialize)]
struct SlackMessage {
    #[serde(rename = "type")]
    kind: String,
    /// "secs.microsecs" — Slack's stable unique ID for a message
    ts: String,
    /// Absent on bot/app messages
    user: Option<String>,
    text: Option<String>,
    /// Present on non-user events: channel_join, bot_message, etc.
    subtype: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackUser {
    id: String,
    name: String,
}
#[derive(Debug, Deserialize)]
struct SlackUserListResponse {
    ok: bool,
    members: Option<Vec<SlackUser>>,
    error: Option<String>,
}

pub struct SlackAdapter {
    client: Client,
    token: String,
    channel_id: String,
}

impl MediaAdapter for SlackAdapter {
    fn get_messages<'a>(
        &'a self,
        last_message_time: &'a Option<DateTime<Utc>>,
    ) -> PinnedAsync<'a, Result<Vec<SaveData>, Box<dyn Error>>> {
        Box::pin(async move {
            let id_username_map = match self.get_id_name_map().await {
                Ok(m) => m,
                Err(e) => {
                    println!("Error while getting slack ID -> username mappings: {}", e);
                    HashMap::new()
                }
            };

            let mut query: Vec<(&str, String)> = vec![
                ("channel", self.channel_id.clone()),
                ("limit", String::from("999")), // max Slack allows per request
            ];

            if let Some(dt) = last_message_time {
                // Slack's `oldest` is *exclusive*: only messages with ts > oldest
                // are returned, so the last saved message won't be duplicated.
                // Format mirrors Slack ts: "unix_secs.microsecs"
                let oldest = format!("{}.{:06}", dt.timestamp(), dt.timestamp_subsec_micros());
                query.push(("oldest", oldest));
            }

            let resp = self
                .client
                .get("https://slack.com/api/conversations.history")
                .bearer_auth(&self.token)
                .query(&query)
                .send()
                .await?
                .json::<SlackHistoryResponse>()
                .await?;

            if !resp.ok {
                let msg = resp.error.unwrap_or("unknown Slack API error".into());
                return Err(msg.into());
            }

            let save_data = resp
                .messages
                .unwrap_or_default()
                .into_iter()
                // Skip system events (joins, leaves) and bot/app messages
                .filter(|m| m.kind == "message" && m.subtype.is_none())
                .map(|m| {
                    let timestamp = parse_slack_ts(&m.ts);
                    let username = m
                        .user
                        .as_ref()
                        .map(|id| id_username_map.get(id).unwrap_or(id));
                    let cleartext_message =
                        substitute_ids(&m.text.unwrap_or_default(), &id_username_map);
                    SaveData {
                        message: MediaMessage::Full {
                            message: cleartext_message,
                        },
                        time: timestamp,
                        author: username.cloned().unwrap_or_default(),
                    }
                })
                .collect();

            Ok(save_data)
        })
    }
}

/// Parses a Slack `ts` string (e.g. `"1512085950.000216"`) into a `DateTime<Utc>`.
fn parse_slack_ts(ts: &str) -> Option<DateTime<Utc>> {
    let (secs_str, micros_str) = ts.split_once('.')?;
    let secs: i64 = secs_str.parse().ok()?;
    let micros: i64 = micros_str.parse().ok()?;
    Utc.timestamp_micros(secs * 1_000_000 + micros).single()
}

fn substitute_ids(input: &str, id_name_map: &HashMap<String, String>) -> String {
    let mut message_reconstructor: Vec<&str> = Vec::new();

    for section in input.split("<@") {
        let Some((identifier, rest)) = section.split_once('>') else {
            message_reconstructor.push(section);
            continue;
        };
        let id = match identifier.split_once('|') {
            Some((id, _fallback)) => id,
            None => identifier,
        };
        let addition = match id_name_map.get(id) {
            Some(username) => username,
            None => id,
        };
        message_reconstructor.extend([addition, rest]);
    }

    message_reconstructor.join("")
}

impl SlackAdapter {
    pub fn new() -> Result<Self, io::Error> {
        let token = env::var("SLACK_APP_TOKEN").expect("Slack App Token not present");
        let channel_id = env::var("SLACK_CHANNEL_ID").expect("Slack Channel ID not present");
        Ok(SlackAdapter {
            client: Client::new(),
            token,
            channel_id,
        })
    }

    async fn get_id_name_map(&self) -> reqwest::Result<HashMap<String, String>> {
        let resp = self
            .client
            .get("https://slack.com/api/users.list")
            .bearer_auth(&self.token)
            .send()
            .await?
            .json::<SlackUserListResponse>()
            .await?;

        if !resp.ok {
            let msg = resp.error.unwrap_or("unknown Slack API error".into());
            println!("Failed to fetch slack users: {}", msg);
            return Ok(HashMap::new());
        }

        Ok(resp
            .members
            .unwrap_or_default()
            .into_iter()
            .map(|u| (u.id, u.name))
            .collect())
    }
}
