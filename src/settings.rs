#[derive(Debug, Clone, serde::Deserialize)]
pub struct Settings {
    #[serde(default = "Settings::default_openmeter_url")]
    pub openmeter_url: String,
    pub openmeter_token: String,

    #[serde(default = "Settings::default_namespace")]
    pub namespace: String,
    #[serde(default = "Settings::default_label_selector")]
    pub label_selector: String,
    #[serde(default = "Settings::default_source")]
    pub source: String,
    #[serde(default = "Settings::default_monitor_rate_seconds")]
    pub monitor_rate_seconds: u64,
}

impl Settings {
    pub fn from_config() -> anyhow::Result<Self> {
        config::Config::builder()
            .add_source(config::Environment::default())
            .build()?
            .try_deserialize::<Self>()
            .map_err(|e| e.into())
    }

    fn default_namespace() -> String {
        "default".to_string()
    }

    fn default_label_selector() -> String {
        "k8s.openmeter.cloud/monitor".to_string()
    }

    fn default_openmeter_url() -> String {
        "https://openmeter.cloud".to_string()
    }

    fn default_source() -> String {
        "kubernetes-api".to_string()
    }

    fn default_monitor_rate_seconds() -> u64 {
        5
    }
}
