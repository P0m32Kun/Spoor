use reqwest::blocking::Client;

pub fn fetch_text(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let resp = client.get(url).send()?.error_for_status()?;
    resp.text()
}
