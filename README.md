# Citoer

Citoer saves your funniest quotes and extracts the quote, who said it, who it was directed at, and who recorded it, to a storage medium of your choice.

- **Supported media platform:** Slack
- **Supported storage medium:** Google Sheets

Citoer can't track emoji reactions or thread replies.

Citoer runs from a GitHub Actions workflow using a prebuilt Docker image (built automatically the first time, if it doesn't exist yet). The workflow is triggered once a day automatically (scheduled using cron).

## Setup

Citoer is built to be **forked** - everyone runs their own copy with their own credentials. More setup, but more privacy and control.

### 1. Fork & enable Actions

1. Click **Fork** at the top of this repo.
2. In your fork, go to **Settings → Actions → General** and select **"Allow all actions and reusable workflows"**.
### 2. Where to add secrets & variables

Throughout this guide you'll be asked to add some values. They all go in the same place: **Settings → Secrets and Variables → Actions**, in your fork.

- **Secrets** tab → for sensitive values (API keys, tokens).
- **Variables** tab → for everything else.
Click **"New repository secret/variable"**, then enter the name (given below) and the value.

### 3. Connect Slack

Set the variable `MEDIA_ADAPTER` to `slack` (optional, it's the default).

1. Go to [api.slack.com/apps](https://api.slack.com/apps) → **Create New App** → **From Scratch**. Name it and pick your workspace.
2. Open **OAuth & Permissions** and add these bot scopes:
   - `channels:history` (also add `groups:history` if your quotes channel is private)
   - `users:read`, turns user IDs into readable names and @'s
3. Install the app to your workspace, then copy the app token → save as secret **`SLACK_APP_TOKEN`**.
4. In your quotes channel, type `/invite @[your app's name]`.
5. Copy the channel ID. It's the last part of the channel URL, or use right-click on channel → **Copy → Copy link** → extract channel ID from clipboard -> save as variable **`SLACK_CHANNEL_ID`**.

### 4. Connect Google Sheets

Set the variable `STORAGE_ADAPTER` to `google_sheets` (optional, it's the default).

1. Enable the [Google Sheets API](https://console.cloud.google.com/apis/api/sheets.googleapis.com/) for a Google Cloud project (create one if you don't have one).
2. Go to the [Credentials page](https://console.cloud.google.com/apis/api/sheets.googleapis.com/credentials) → **Create Credentials → Service Account**. Give it a name and the **Editor** role.
3. Note the service account's **email**, and download its **JSON key file**.
4. Save the JSON file's contents as secret **`GOOGLE_SERVICE_ACCOUNT`**.
5. Open (or create) the Google Sheet you want to use, and share it with the service account email as an **Editor**.
6. Copy the spreadsheet ID from the URL (`.../spreadsheets/d/<ID>/edit`) → save as variable **`GOOGLE_SHEETS_SPREADSHEET_ID`**.
7. Save the name of the sheet/tab as variable **`GOOGLE_SHEETS_PAGE_NAME`**.

### Quick reference

| Name | Type | Value |
|---|---|---|
| `MEDIA_ADAPTER` | Variable | `slack` *(default)* |
| `SLACK_APP_TOKEN` | Secret | Your Slack app's token |
| `SLACK_CHANNEL_ID` | Variable | Your quotes channel's ID |
| `STORAGE_ADAPTER` | Variable | `google_sheets` *(default)* |
| `GOOGLE_SERVICE_ACCOUNT` | Secret | Contents of your service account's JSON key |
| `GOOGLE_SHEETS_SPREADSHEET_ID` | Variable | ID from your sheet's URL |
| `GOOGLE_SHEETS_PAGE_NAME` | Variable | Name of the sheet/tab |
| `QUOTE_REGEX` | Variable | Custom format *(optional, see below)* |

## Custom quote format

By default, citoer reads messages in this (Swedish) format (with some leniency):

```
"[quote]" - @[quotee] till @[receiver]
```

Quotation marks, `@`, and the `till @[receiver]` part are all optional.

If a quote does not match this format, the entire message will be saved as the "quote", and other values will remain "untracked".

Want a different language or layout? Set your own `QUOTE_REGEX` variable. It must keep the capture groups `text`, `quotee`, and (optionally) `receiver`.

Default regex:
```re
(?P<quote1>["\"”])?(?P<text>.*?)(?P<quote2>["\"”])?\s*-\s*@?(?P<quotee>.*?)(?P<till>\s+till\s+@(?P<receiver>.*))?$
```
