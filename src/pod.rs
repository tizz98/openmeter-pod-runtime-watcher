use std::collections::HashMap;

use cloudevents::{Event, EventBuilder, EventBuilderV10};
use k8s_openapi::api::core::v1::Pod;

use crate::openmeter;

#[derive(Debug)]
pub struct PodInfo {
    pub monitor: String,
    pub subject: String,
    pub data: HashMap<String, String>,
    pub source: String,
}

impl PodInfo {
    pub async fn send_to_openmeter(
        self,
        openmeter_api: &openmeter::OpenMeterAPI,
    ) -> anyhow::Result<()> {
        if self.source == "" {
            anyhow::bail!("PodInfo source must be set");
        }

        let event: Event = self.try_into()?;
        openmeter_api.send(event).await?;
        Ok(())
    }

    pub fn with_duration(&mut self, duration: u64) {
        self.data
            .insert("duration".to_string(), duration.to_string());
    }

    pub fn with_source(&mut self, source: String) {
        self.source = source;
    }
}

impl From<Pod> for PodInfo {
    fn from(value: Pod) -> Self {
        // Parse data from all k8s.opemeter.cloud/data-* labels
        let data = value
            .metadata
            .labels
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|(k, _)| k.starts_with("k8s.openmeter.cloud/data-"))
            .map(|(k, v)| (k.replace("k8s.openmeter.cloud/data-", ""), v.to_string()))
            .collect();

        PodInfo {
            monitor: value
                .metadata
                .labels
                .clone()
                .unwrap_or_default()
                .get("k8s.openmeter.cloud/monitor")
                .unwrap_or(&"unknown".to_string())
                .to_string(),
            subject: value
                .metadata
                .labels
                .unwrap_or_default()
                .get("k8s.openmeter.cloud/subject")
                .unwrap_or(&"unknown".to_string())
                .to_string(),
            data,
            source: "".to_string(),
        }
    }
}

impl TryInto<Event> for PodInfo {
    type Error = anyhow::Error;

    fn try_into(self) -> anyhow::Result<Event> {
        EventBuilderV10::new()
            .id(uuid::Uuid::new_v4().to_string())
            .ty(self.monitor)
            .source(self.source)
            .subject(self.subject)
            .data("application/json", serde_json::to_value(self.data)?)
            .build()
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use cloudevents::AttributesReader;

    use super::*;

    #[test]
    fn test_podinfo_from_pod() {
        let pod = Pod {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                labels: Some(BTreeMap::from([
                    (
                        "k8s.openmeter.cloud/monitor".to_string(),
                        "test-monitor".to_string(),
                    ),
                    (
                        "k8s.openmeter.cloud/subject".to_string(),
                        "test-subject".to_string(),
                    ),
                    (
                        "k8s.openmeter.cloud/data-test".to_string(),
                        "test-value".to_string(),
                    ),
                    (
                        "k8s.openmeter.cloud/data-foo-bar-baz".to_string(),
                        "any valid value here".to_string(),
                    ),
                ])),
                ..Default::default()
            },
            ..Default::default()
        };

        let pod_info = PodInfo::from(pod);
        assert_eq!(pod_info.monitor, "test-monitor");
        assert_eq!(pod_info.subject, "test-subject");
        assert_eq!(pod_info.data.get("test").unwrap(), "test-value");
        assert_eq!(
            pod_info.data.get("foo-bar-baz").unwrap(),
            "any valid value here"
        );
    }

    #[test]
    fn test_podinfo_try_into_event() {
        let mut pod_info = PodInfo {
            monitor: "test-monitor".to_string(),
            subject: "test-subject".to_string(),
            data: [("test".to_string(), "test-value".to_string())]
                .iter()
                .cloned()
                .collect(),
            source: "kubernetes-api-modified".to_string(),
        };
        pod_info.with_duration(10);

        let event: Event = pod_info.try_into().unwrap();
        assert_eq!(event.id(), event.id());
        assert_eq!(event.ty(), "test-monitor");
        assert_eq!(event.subject(), Some("test-subject"));
        assert_eq!(event.source(), "kubernetes-api-modified");
        assert_eq!(event.datacontenttype(), Some("application/json"));
        assert_eq!(
            event.data().unwrap(),
            &cloudevents::Data::Json(serde_json::json!({
                "test": "test-value",
                "duration": "10"
            }))
        );
    }
}
