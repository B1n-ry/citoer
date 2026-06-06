mod adapters;

use adapters::GroupedData;
use dotenv;
use regex::Regex;
use std::env;

const QUOTE_REGEX: &str = r#"(?P<quote1>["\"”])?(?P<text>.*?)(?P<quote2>["\"”])?\s*-\s*@?(?P<quotee>.*?)(?P<till>\s+till\s+@(?P<receiver>.*))?$"#;

fn main() {
    dotenv::dotenv().ok();

    let message = env::var("MESSAGE").expect("MESSAGE env var required");
    let quoter = env::var("QUOTER").expect("QUOTER env var required");

    let Some(ungrouped_message) = ungroup(&message) else {
        println!("Error: Invalid format on message '{}'", message);
        return;
    };

    let ungrouped_message = GroupedData {
        quoter: &quoter,
        ..ungrouped_message
    };

    let adapter_name = env::var("STORAGE").unwrap_or("google_sheets".to_string());
    let adapter = adapters::get_adapter(&adapter_name)
        .expect(&format!("No adapter found for '{}'", adapter_name));

    let save_result = adapter.save(&ungrouped_message);
    match save_result {
        Err(err) => {
            println!("Error: {}", err);
        }
        Ok(_) => {
            println!("Successfully updated data!");
        }
    };
}

fn ungroup(message: &str) -> Option<GroupedData> {
    let regex = Regex::new(QUOTE_REGEX).ok()?;

    let caps = regex.captures(message)?;
    let groups = GroupedData {
        quote: caps.name("text")?.as_str(),
        quotee: caps.name("quotee")?.as_str(),
        quoter: "",
        receiver: caps.name("receiver").map(|m| m.as_str()),
    };

    Some(groups)
}
