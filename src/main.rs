mod openmeter;
mod pod;
mod settings;

use anyhow::Context;
use k8s_openapi::api::core::v1::Pod;
use kube::api::Api;
use kube::Client;
use tokio::time;
use tracing::{debug, error, info};

use crate::pod::PodInfo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let settings = settings::Settings::from_config().context("Unable to load settings")?;
    let openmeter_api = openmeter::OpenMeterAPI::new(openmeter::OpenMeterAPIConfig {
        url: settings.openmeter_url,
        token: settings.openmeter_token,
    })?;

    // Based on settings.monitor_rate_seconds, loop and fetch all pods matching the settings.label_selector and field status.phase==Running
    // For each pod, parse the data from the labels and send an event to OpenMeter
    let client = Client::try_default()
        .await
        .context("Unable to load k8s client from defaults")?;
    let pods_api: Api<Pod> = Api::namespaced(client, settings.namespace.as_str());

    info!(
        interval = format!("{}s", settings.monitor_rate_seconds),
        "Starting pod monitor"
    );

    let mut interval = time::interval(time::Duration::from_secs(settings.monitor_rate_seconds));
    loop {
        interval.tick().await;

        let pods = pods_api
            .list(
                &kube::api::ListParams::default()
                    .labels(&settings.label_selector)
                    .fields("status.phase=Running"),
            )
            .await?;

        match pods.items.is_empty() {
            true => debug!("No running pods found"),
            false => debug!(pods_found = pods.items.len(), "Found matching pods"),
        }

        // Spawn a task for each pod to send the event to OpenMeter
        for pod in pods.items {
            let mut pod_info = PodInfo::from(pod);
            pod_info.with_duration(settings.monitor_rate_seconds);
            pod_info.with_source(settings.source.clone());

            let openmeter_api = openmeter_api.clone();
            tokio::spawn(async move {
                let monitor = pod_info.monitor.clone();
                let subject = pod_info.subject.clone();
                let data = pod_info.data.clone();

                debug!(
                    monitor = monitor,
                    subject = subject,
                    data = format!("{:?}", data),
                    "Sending pod info to OpenMeter"
                );

                match pod_info.send_to_openmeter(&openmeter_api).await {
                    Ok(_) => info!(
                        monitor = monitor,
                        subject = subject,
                        "Sent pod info to OpenMeter"
                    ),
                    Err(e) => error!(
                        monitor = monitor,
                        subject = subject,
                        error = e.to_string(),
                        "Failed to send pod info to OpenMeter"
                    ),
                }
            });
        }
    }
}
