# Citoer

This app is meant to save all your funniest quotes to a centralized storage medium.

Citoer makes use of GitHub actions to save quotes. When the project is built a docker container is stored to GitHub Container Registry (GHCR). When the run workflow is triggered, which it can be by `workflow_dispatch` using GitHub's REST API, it runs using the pre-compiled docker image. If no image exists it will build it first.

So far citoer supports the following storage mediums:

- Google sheets

Citoer has the power to track the quotes themselves, the person quoting someone, if the quote was directed to/at someone else, and finally who made the quote. These values are tracked individually, and depending on how they are stored filters or sorting can be applied for a user-friendly representation of the data.

Citoer can NOT keep track of emoji-reactions to quotes, or replies in threads.

So far citoer supports the following media platform:

- Slack

## Setup

The idea behind citoer is for people to fork the repository, and then build their own implementation of citoer using their own credentials and variables. This makes the setup slightly more complicated, but increases scalability and increases privacy for each user where they are in control of their environment, instead of all requests going to a central server.

### GitHub Configuration

What you need to do is to first off fork the repository. This can be done by clicking the "fork" button at the top of the main repository page. All following mentions of "repository" will hereby refer to the fork you made, not the original repository (unless explicitly stated). When you have forked the original repository to your own user, make sure GitHub Actions are enabled. This is done by going to `Settings -> Actions -> General` in your repository. Make sure "Allow all actions and reusable workflows" is selected. (Note that there may be ways to not require this exact option. The expertise behind citoer is limited in how configurations like this works in GitHub).

### GitHub variable management

Since we need to connect to a number of external parties we will need to keep track of some API-keys and other configurable variables. These we keep stored in GitHub. In this guide we will refer to either "GitHub secrets" or "GitHub variables". These are located in your repository, and then navigating to `Settings -> Secrets and Variables (left column) -> Actions`. Here we have two tabs. One called "Secrets" and the other called "Variables". When asked to add variables and secrets, go here and press the button "New repository secret/variable" (depending on what you add). Then you fill in the name of the variable (name will be mentioned by this guide), and its value (will be chosen by you, or copied from a third party).

### Setting up media integration

To set up citoer you need to decide on a media platform (choose an option to jump to its part in the setup guide).

- [Slack](#slack-integration)

#### Slack Integration

If you've chosen Slack as your media platform, set the [GitHub variable](#github-variable-management) `MEDIA_ADAPTER` to `slack` (this is optional, as this is the default).

In order to read messages from Slack in order to store them somewhere, we need to connect a Slack app/bot to citoer. First, make sure you are signed in to the Slack workspace where your quotes-channel is located. Then, navigate to https://api.slack.com/apps. Click "Create New App" and select the "From Scratch" alternative. Give the app a name (will only affect what it appears as in Slack), and select the workspace you want it having access to. Click "Create App". This will create the Slack app.

Next, in the sidebar to the left, click "OAuth & Permissions". Here, add the following bot permissions: `channels:history` (if you intend to use citoer in a private channel, also add `groups:history`), and `users:read`. The former will give the Slack app access to read your channel messages, and the latter will replace user IDs in Slack messages with interpretable identifiers (includes making @'s readable, since when fetching responses from Slack, @'s are just the user ID). After permissions has been assigned, you can add the Slack app to your workspace! When this is done you will get an app token from where you configured the permissions. Copy this and add it as a [GitHub secret](#github-variable-management) with the name `SLACK_APP_TOKEN`.

Next, navigate to your quote-channel in your slack workspace. Write in the chat `/invite @[Slack app name]`, and the app will be added to the channel. Then, you will need to copy the Slack channel ID. This can be done either (if you run Slack from a browser) by finding the channel ID in your Slack URL when located in the channel (the part after the last '/'), or by right-clicking on your channel, selecting `Copy -> Copy link`, and then pasting it to then only copy the channel ID (again, the part after the last '/').

Add the channel ID as a [GitHub variable](#github-variable-management) with the name `SLACK_CHANNEL_ID`.

You're now done with the Slack configuration!

### Setting up quote storage

To set up citoer you need to decide on a storage medium (choose an option to jump to its part in the setup guide).

- [Google Sheets](#google-sheets)

#### Google Sheets

If you've chosen Slack as your storage medium, set the [GitHub variable](#github-variable-management) `STORAGE_ADAPTER` to `google_sheets` (this is optional, as this is the default).

To store data to a sheet document, you need to set up a Google Service Account in order to automatically interact with the document. For this you first need to enable Google Sheets API in a project in the [Google Cloud Console Dashboard](https://console.cloud.google.com/apis/api/sheets.googleapis.com/). If you do not have a project you want to use you can create a new one.

When you have enabled Google Sheets API, go to the [Google Sheets Credentials page](https://console.cloud.google.com/apis/api/sheets.googleapis.com/credentials). Here, click "Create Credentials", and then "Service Account". Enter a name for the service account, and continue. Make sure to give the service account the "Editor" role under permissions. From here you should be able to first off get an email for the service account (save this), and also be able to download a key file, save that one as well.

After you have your service account JSON file, add its contents as a [GitHub secret](#github-variable-management) with the name `GOOGLE_SERVICE_ACCOUNT`.

To connect to a Google sheet, simply create a new one or navigate to an already existing sheet. Here, add the service account email as an editor of the document. Also make sure to copy the ID of the spreadsheet. This is found in the URL: `https://docs.google.com/spreadsheets/d/<SPREADSHEET_ID>/edit`. You also need to take note of the sheet/page name where you want the quotes to be stored. Add the spreadsheet ID as a [GitHub variable](#github-variable-management) with the name `GOOGLE_SHEETS_SPREADSHEET_ID`, and the sheet/page name as a [GitHub variable](#github-variable-management) with the name `GOOGLE_SHEETS_PAGE_NAME`.

We've now successfully connected Google Sheets to our citoer instance!

### Custom quote formats

The format to decide how sections of the quote message are decoded is handled by a regex. This regex is by default

```re
(?P<quote1>["\"”])?(?P<text>.*?)(?P<quote2>["\"”])?\s*-\s*@?(?P<quotee>.*?)(?P<till>\s+till\s+@(?P<receiver>.*))?$
```

You can define your own by adding the [GitHub variable](#github-variable-management) `QUOTE_REGEX`. Unless you want to go edit the code, the capture groups "text", "quotee", and "receiver" (optional) must be present. These are the capture groups used for the quote itself, the person who was quoted, and if the quote was directed towards someone (and in that case who).
