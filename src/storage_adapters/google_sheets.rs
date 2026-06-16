extern crate google_sheets4 as sheets4;
extern crate hyper;
use chrono::{DateTime, Utc};
use google_sheets4::api::{BatchUpdateValuesRequest, ValueRange};
use serde_json::Value;
use sheets4::{hyper_rustls, hyper_util, Sheets};
use std::{collections::HashMap, env, error::Error, io, iter};
use yup_oauth2::{self, ServiceAccountAuthenticator};

use crate::{MediaMessage, PinnedAsync, SaveData};

use super::StorageAdapter;

pub struct GoogleSheetsAdapter {
    spreadsheet_id: String,
    sheet_name: String,
    hub: Sheets<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

#[repr(u8)]
#[derive(PartialEq, Eq, Hash)]
enum StorageColumn {
    Quote = b'A',
    Quotee = b'B',
    Receiver = b'C',
    Quoter = b'D',
    Time = b'E',
}

impl From<StorageColumn> for char {
    fn from(value: StorageColumn) -> Self {
        value as u8 as char
    }
}

// Type of START_ROW can be changed freely
const START_ROW: u16 = 2;

fn get_range_row_bounds(range: &str) -> Option<(usize, usize)> {
    let range = range.split('!').next_back()?; // We can allow for '!' to be in the sheet name, but it can't be in the column or row section of the range
    let (start, end) = range.split_once(':')?;
    let start_row: usize = start.trim_start_matches(char::is_alphabetic).parse().ok()?;
    let end_row: usize = end.trim_start_matches(char::is_alphabetic).parse().ok()?;

    Some((start_row, end_row))
}

impl GoogleSheetsAdapter {
    pub async fn new() -> Result<Self, io::Error> {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");

        let spreadsheet_id =
            env::var("GOOGLE_SHEETS_SPREADSHEET_ID").expect("Spreadsheet ID not present");
        let sheet_name =
            env::var("GOOGLE_SHEETS_PAGE_NAME").expect("No page name for google sheets found");
        let google_credentials = env::var("GOOGLE_SERVICE_ACCOUNT")
            .expect("No google service account credentials provided");

        let secret: yup_oauth2::ServiceAccountKey =
            yup_oauth2::parse_service_account_key(&google_credentials)?;

        let auth = ServiceAccountAuthenticator::builder(secret).build().await?;

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(
                    hyper_rustls::HttpsConnectorBuilder::new()
                        .with_native_roots()?
                        .https_or_http()
                        .enable_http2()
                        .build(),
                );

        let hub = Sheets::new(client, auth);
        Ok(Self {
            spreadsheet_id,
            sheet_name,
            hub,
        })
    }
    async fn get_quote_rows(&self) -> HashMap<String, usize> {
        let quote_range = format!(
            "{}!{}{}:{}",
            self.sheet_name,
            char::from(StorageColumn::Quote),
            START_ROW,
            char::from(StorageColumn::Quote),
        );
        let quotee_range = format!(
            "{}!{}{}:{}",
            self.sheet_name,
            char::from(StorageColumn::Quotee),
            START_ROW,
            char::from(StorageColumn::Quotee),
        );
        let (_response, batch_ranges) = match self
            .hub
            .spreadsheets()
            .values_batch_get(&self.spreadsheet_id)
            .add_ranges(&quote_range)
            .add_ranges(&quotee_range)
            .major_dimension("COLUMNS")
            .doit()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to fetch IDs in sheet: {}", e);
                return HashMap::new();
            }
        };
        let Some([quote_values, quotee_values]) = batch_ranges.value_ranges.as_deref() else {
            // No data was found
            return HashMap::new();
        };

        let quote_range_first_row = quote_values
            .range
            .as_ref()
            .and_then(|r| get_range_row_bounds(r).map(|(start, _end)| start));
        let quotee_range_first_row = quotee_values
            .range
            .as_ref()
            .and_then(|r| get_range_row_bounds(r).map(|(start, _end)| start));

        let Some(start_ranges) = quote_range_first_row.zip(quotee_range_first_row) else {
            println!(
                "Could not identify ranges from google sheets response when fetching message cache"
            );
            return HashMap::new();
        };

        // Get Option references so we don't require clone or copy
        let (quote_values, quotee_values) =
            (quote_values.values.as_ref(), quotee_values.values.as_ref());
        let (Some(quotes), Some(quotees)) = (
            quote_values.and_then(|v| v.first()),
            quotee_values.and_then(|v| v.first()),
        ) else {
            // Values not found (empty spreadsheet?)
            return HashMap::new();
        };

        let quotee_iter = quotees.iter().map(|q| q.as_str());

        // Quotes are THE most important. If some message does not have a quotee for some reason, don't dupe them.
        // We can here convert quotee list/iterator into optional strings, and chaining with `None`, guaranteeing
        // that the quotes will be the limiting factor. We combine this with the knowledge of which row each column
        // starts at, and using this we determine how the quotee iterator should look
        let quotee_iter: Box<dyn Iterator<Item = Option<&str>>> = match start_ranges {
            (a, b) if a < b => Box::new(iter::repeat_n(None, b - a).chain(quotee_iter))
                as Box<dyn Iterator<Item = Option<&str>>>,
            (a, b) if a > b => Box::new(quotee_iter.skip(a - b).chain(iter::repeat(None)))
                as Box<dyn Iterator<Item = Option<&str>>>,
            _ => Box::new(quotee_iter.chain(iter::repeat(None)))
                as Box<dyn Iterator<Item = Option<&str>>>,
        };

        quotes
            .iter()
            .zip(quotee_iter)
            .enumerate()
            .filter_map(|(i, (quote, quotee))| Some((i, quote.as_str()?, quotee)))
            .map(|(i, quote, quotee)| {
                (
                    format!("{}::{}", quote, quotee.unwrap_or("")),
                    // This should never revert to START_ROW since we know 'quote_range_first_row' is not None
                    i + quote_range_first_row.unwrap_or(START_ROW as usize),
                )
            })
            .collect()
    }
}

impl StorageAdapter for GoogleSheetsAdapter {
    fn save<'a>(&'a self, data: &'a [SaveData]) -> PinnedAsync<'a, Result<(), Box<dyn Error>>> {
        Box::pin(async move {
            let quote_rows = self.get_quote_rows().await;

            let mut quotes: Vec<Value> = Vec::new();
            let mut quotees: Vec<Value> = Vec::new();
            let mut quoters: Vec<Value> = Vec::new();
            let mut receivers: Vec<Value> = Vec::new();
            let mut times: Vec<Value> = Vec::new();

            data.iter().cloned().for_each(|d| {
                let [quote, quotee, receiver]: [Value; 3] = match d.message {
                    MediaMessage::Full { message } => {
                        [Value::String(message), Value::Null, Value::Null]
                    }
                    MediaMessage::Grouped {
                        quote,
                        quotee,
                        receiver,
                    } => [
                        Value::String(quote),
                        Value::String(quotee),
                        receiver.map_or(Value::Null, Value::String),
                    ],
                };
                if quote_rows.contains_key(&format!("{}::{}", quote, quotee)) {
                    return; // Avoid duplicates
                }
                quotes.push(quote);
                quotees.push(quotee);
                receivers.push(receiver);
                quoters.push(Value::String(d.author));
                times.push(
                    d.time
                        .map_or(Value::Null, |date| Value::String(date.to_string())),
                );
            });

            let first_free_row = quote_rows
                .values()
                .max()
                .map_or(START_ROW as usize, |i| *i + 1);

            let req = BatchUpdateValuesRequest {
                data: Some(vec![
                    ValueRange {
                        values: Some(vec![quotes]),
                        range: Some(format!(
                            "{}!{}{}:{}",
                            self.sheet_name,
                            char::from(StorageColumn::Quote),
                            first_free_row,
                            char::from(StorageColumn::Quote),
                        )),
                        major_dimension: Some(String::from("COLUMNS")),
                    },
                    ValueRange {
                        values: Some(vec![quotees]),
                        range: Some(format!(
                            "{}!{}{}:{}",
                            self.sheet_name,
                            char::from(StorageColumn::Quotee),
                            first_free_row,
                            char::from(StorageColumn::Quotee),
                        )),
                        major_dimension: Some(String::from("COLUMNS")),
                    },
                    ValueRange {
                        values: Some(vec![receivers]),
                        range: Some(format!(
                            "{}!{}{}:{}",
                            self.sheet_name,
                            char::from(StorageColumn::Receiver),
                            first_free_row,
                            char::from(StorageColumn::Receiver),
                        )),
                        major_dimension: Some(String::from("COLUMNS")),
                    },
                    ValueRange {
                        values: Some(vec![quoters]),
                        range: Some(format!(
                            "{}!{}{}:{}",
                            self.sheet_name,
                            char::from(StorageColumn::Quoter),
                            first_free_row,
                            char::from(StorageColumn::Quoter),
                        )),
                        major_dimension: Some(String::from("COLUMNS")),
                    },
                    ValueRange {
                        values: Some(vec![times]),
                        range: Some(format!(
                            "{}!{}{}:{}",
                            self.sheet_name,
                            char::from(StorageColumn::Time),
                            first_free_row,
                            char::from(StorageColumn::Time),
                        )),
                        major_dimension: Some(String::from("COLUMNS")),
                    },
                ]),
                value_input_option: Some(String::from("RAW")),
                ..Default::default()
            };

            let result = self
                .hub
                .spreadsheets()
                .values_batch_update(req, &self.spreadsheet_id)
                .doit()
                .await;

            result
                .map(|_| ())
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        })
    }

    fn get_most_recent_time<'a>(&'a self) -> PinnedAsync<'a, Option<DateTime<Utc>>> {
        Box::pin(async move {
            let time_letter = char::from(StorageColumn::Time);

            let range = format!(
                "{}!{}{}:{}",
                self.sheet_name, time_letter, START_ROW, time_letter
            );
            let (_response, value_range) = self
                .hub
                .spreadsheets()
                .values_get(&self.spreadsheet_id, &range)
                .doit()
                .await
                .inspect_err(|e| println!("Failed to fetch all times: {}", e))
                .ok()?;

            let values = value_range.values?;
            let mapped_vals: Vec<DateTime<Utc>> = values
                .iter()
                .filter_map(|v| {
                    let time_str = v.first()?.as_str()?;
                    time_str
                        .parse()
                        .inspect_err(|e| println!("Failed to parse {}: {}", &time_str, e))
                        .ok()
                })
                .collect();

            mapped_vals.into_iter().max()
        })
    }
}
