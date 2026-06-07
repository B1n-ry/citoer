extern crate google_sheets4 as sheets4;
extern crate hyper;
use google_sheets4::api::ValueRange;
use serde_json::Value;
use sheets4::{hyper_rustls, hyper_util, Sheets};
use std::{env, future::Future, pin::Pin};
use yup_oauth2::{self, ServiceAccountAuthenticator};

use super::{GroupedData, StorageAdapter};

pub struct GoogleSheetsAdapter {
    spreadsheet_id: String,
    sheet_name: String,
    hub: Sheets<hyper_rustls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>>,
}

impl GoogleSheetsAdapter {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
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
}

impl StorageAdapter for GoogleSheetsAdapter {
    fn save<'a>(
        &'a self,
        data: &'a GroupedData<'a>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
            let &GroupedData {
                quote,
                quoter,
                quotee,
                receiver,
            } = data;

            // Create a ValueRange with the data to append
            let mut req = ValueRange::default();
            req.values = Some(vec![vec![
                Value::from(quote),
                Value::from(quotee),
                Value::from(receiver.unwrap_or("")), // If left as optional and it's None, next cell will misalign
                Value::from(quoter),
            ]]);

            let result = self
                .hub
                .spreadsheets()
                .values_append(req, &self.spreadsheet_id, &self.sheet_name)
                .value_input_option("USER_ENTERED")
                .doit()
                .await;

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to append to sheet: {}", e)),
            }
        })
    }
}
