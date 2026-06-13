extern crate google_sheets4 as sheets4;
extern crate hyper;
use chrono::{DateTime, Utc};
use google_sheets4::api::{BatchUpdateSpreadsheetRequest, Scope, ValueRange};
use serde_json::Value;
use sheets4::{hyper_rustls, hyper_util, Sheets};
use std::{collections::HashMap, env, error::Error, future::Future, io, pin::Pin};
use yup_oauth2::{self, ServiceAccountAuthenticator};

use crate::{MediaMessage, SaveData};

use super::StorageAdapter;

pub struct GoogleSheetsAdapter {
    spreadsheet_id: String,
    sheet_name: String,
    hub: Sheets<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
    column_config: HashMap<StorageColumn, char>,
}

#[derive(PartialEq, Eq, Hash)]
enum StorageColumn {
    QUOTE,
    QUOTEE,
    RECEIVER,
    QUOTER,
    ID,
    TIME,
}

// Type of START_ROW can be changed freely
const START_ROW: u16 = 2;

const COLUMN_CONFIG: [(StorageColumn, char); 6] = [
    (StorageColumn::QUOTE, 'A'),
    (StorageColumn::QUOTEE, 'B'),
    (StorageColumn::RECEIVER, 'C'),
    (StorageColumn::QUOTER, 'D'),
    (StorageColumn::TIME, 'E'),
    (StorageColumn::ID, 'F'),
];

impl GoogleSheetsAdapter {
    pub async fn new() -> Result<Self, io::Error> {
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
            column_config: HashMap::from(COLUMN_CONFIG),
        })
    }
    async fn get_id_rows(&self) -> HashMap<String, usize> {
        let column_name = self
            .column_config
            .get(&StorageColumn::ID)
            .expect("Missing ID in configuration");
        let range = format!(
            "{}!{}{}:{}",
            self.sheet_name, column_name, START_ROW, column_name,
        );

        let (_response, value_range) = match self
            .hub
            .spreadsheets()
            .values_get(&self.spreadsheet_id, &range)
            .doit()
            .await
        {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to fetch IDs in sheet: {}", e);
                return HashMap::new();
            }
        };
        let Some(values) = value_range.values else {
            // No IDs were found at all, presumably due to an empty sheet
            return HashMap::new();
        };
        values
            .iter()
            .enumerate()
            .filter_map(|(i, row)| {
                row.first()
                    .map(|id| (id.to_string(), i + START_ROW as usize))
            })
            .collect()
    }
}

impl StorageAdapter for GoogleSheetsAdapter {
    fn save<'a>(
        &'a self,
        data: &Vec<SaveData>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send + 'a>> {
        Box::pin(async move {
            let MediaMessage::Grouped {
                quote,
                quotee,
                receiver,
            } = &data.message
            else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Supplied message was not grouped up",
                ))
                .map_err(|e| Box::new(e) as Box<dyn Error>);
            };

            // If left as optional and it's None, next cell will misalign
            let receiver: &str = receiver.as_ref().map_or("", |s| s.as_str());

            // Create a ValueRange with the data to append
            /* let mut req = ValueRange::default();
            req.values = Some(vec![vec![
                Value::from(quote.as_str()),
                Value::from(quotee.as_str()),
                Value::from(receiver),
                Value::from(data.author.as_str()),
            ]]);

            let result = self
                .hub
                .spreadsheets()
                .values_append(req, &self.spreadsheet_id, &self.sheet_name)
                .value_input_option("USER_ENTERED")
                .doit()
                .await;

            result
                .map(|_| ())
                .map_err(|e| Box::new(e) as Box<dyn Error>) */

            let mut req = BatchUpdateSpreadsheetRequest::default();
            todo!("Make dynamic stores using batch updates");
        })
    }

    fn get_most_recent_time<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Option<DateTime<Utc>>> + Send + 'a>> {
        Box::pin(async move {
            let time_letter = self.column_config.get(&StorageColumn::TIME)?;

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
                .inspect_err(|e| println!("{}", e))
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

            let most_recent = mapped_vals.iter().max();

            most_recent.map(|&t| t)
        })
    }
}
