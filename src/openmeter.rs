use cloudevents::AttributesReader;
use tracing::debug;

#[derive(Clone)]
pub struct OpenMeterAPI {
    client: reqwest::Client,
    config: OpenMeterAPIConfig,
}

#[derive(Clone)]
pub struct OpenMeterAPIConfig {
    pub url: String,
    pub token: String,
}

impl OpenMeterAPI {
    pub fn new(config: OpenMeterAPIConfig) -> anyhow::Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", config.token).parse()?);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;
        Ok(OpenMeterAPI { client, config })
    }

    pub async fn send(&self, event: cloudevents::Event) -> anyhow::Result<()> {
        // TODO: use self.client.post(url).event(event)
        // Currently a bug with setting the content-type header.
        // See: https://github.com/cloudevents/sdk-rust/issues/214
        let url = format!("{}/api/v1/events", self.config.url);
        let resp = self
            .client
            .post(url)
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/cloudevents+json",
            )
            .json(&event)
            .send()
            .await?;

        if !resp.status().is_success() {
            let resp_content = resp
                .text()
                .await
                .unwrap_or("<no response content>".to_string());
            anyhow::bail!("Failed to send event to openmeter:\n{}", resp_content);
        }

        debug!(event_id = event.id(), "Sent event to openmeter");
        Ok(())
    }
}
