# Citoer

This app is meant to save all your funniest quotes to a centralized storage medium.

Citoer makes use of GitHub actions to save quotes. When the project is built a docker container is stored to GitHub Container Registry (GHCR). When the run workflow is triggered, which it can be by `workflow_dispatch` using GitHub's REST API, it runs using the pre-compiled docker image. If no image exists it will build it first.

So far citoer supports the following storage mediums:

- Google sheets

Citoer has the power to track the quotes themselves, the person quoting someone, if the quote was directed to/at someone else, and finally who made the quote. These values are tracked individually, and depending on how they are stored filters or sorting can be applied for a user-friendly representation of the data.

Citoer can NOT keep track of reactions to quotes in slack.

## Setup

The idea behind citoer is for people to fork the repository, and then build their own implementation of citoer using their own credentials and variables. This makes the setup slightly more complicated, but increases scalability and increases privacy for each user where they are in control of their environment, instead of all requests going to a central server.

### GitHub Configuration

What you need to do is to first off fork the repository. This can be done by clicking the "fork" button at the top of the main repository page. All following mentions of "repository" will hereby refer to the fork you made, not the original repository (unless explicitly stated). When you have forked the original repository to your own user, make sure GitHub Actions are enabled. This is done by going to `Settings -> Actions -> General` in your repository. Make sure "Allow all actions and reusable workflows" is selected. (Note that there may be ways to not require this exact option. The expertise behind citoer is limited in how configurations like this works in GitHub).

### Slack Integration

We also need to connect slack to our GitHub actions. For this we need a GitHub PAT (Personal Access Token). Go to your [GitHub account page for Fine-grained access tokens](https://github.com/settings/personal-access-tokens). Here, click "Generate new token". Give your token a name and a description if you want to. Choose an expiration date (after this date (if set) the slack interaction will no longer work). Under "Repository access", make sure the PAT has access to your forked repository. Give the PAT permissions "Read and Write access to actions". Next, click "Generate token" and copy the token generated.

### Setting up quote storage

To set up citoer you need to decide on a storage medium (choose an option to jump to its part in the setup guide).

- [Google Sheets](#google-sheets)

#### Google Sheets

To store data to a sheet document, you need to set up a Google Service Account in order to automatically interact with the document. For this you first need to enable Google Sheets API in a project in the [Google Cloud Console Dashboard](https://console.cloud.google.com/apis/api/sheets.googleapis.com/). If you do not have a project you want to use you can create a new one.

When you have enabled Google Sheets API, go to the [Google Sheets Credentials page](https://console.cloud.google.com/apis/api/sheets.googleapis.com/credentials). Here, click "Create Credentials", and then "Service Account". Enter a name for the service account, and continue. Make sure to give the service account the "Editor" role under permissions. From here you should be able to first off get an email for the service account (save this), and also be able to download a key file, save that one as well.

After you have your service account JSON file, go to your forked GitHub repository, and navigate to `Settings -> Secrets and Variables (left column) -> Actions`. Here, click "New repository secret". The name of this repository secret should be `GOOGLE_SERVICE_ACCOUNT`, and the value/secret should be the contents of the JSON file you downloaded.

To connect to a Google sheet, simply create a new one or navigate to an already existing sheet. Here, add the service account email as an editor of the document. Also make sure to copy the ID of the spreadsheet. This is found in the URL: `https://docs.google.com/spreadsheets/d/<SPREADSHEET_ID>/edit`. You also need to take note of the sheet/page name where you want the quotes to be stored.

When you have your spreadsheet ID and sheet name, go back to your forked repository and again navigate to `Settings -> Secrets and Variables (left column) -> Actions`. This time, click the tab "Variables" on this page. Now add the two values as repository variables like you did with the service account JSON file. Call the spreadsheet ID `GOOGLE_SHEETS_SPREADSHEET_ID` and the sheet name `GOOGLE_SHEETS_PAGE_NAME`.

We've now successfully connected Google Sheets to our citoer instance!
