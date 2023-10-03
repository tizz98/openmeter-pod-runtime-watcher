# OpenMeter Pod Runtime Watcher

A simple program to watch pods in a Kubernetes cluster and report their lifetime to [OpenMeter](https://openmeter.io/).

This application is meant to be run using [Helm](https://helm.sh/), but you can test it directly on your machine using the binary build.

## Getting started

### Prerequisites

1. You have a Kubernetes cluster running
2. You have a running OpenMeter instance (cloud or self-hosted)
3. You have an API key from OpenMeter

### Helm (recommended)

[See the helm chart README](./charts/openmeter-pod-runtime-watcher/README.md).

### Local (testing only)

When running locally, this assumes you have a Kubernetes context already configured.

1. Set your environment variables
    
    ```bash
    export OPENMETER_TOKEN=om_1234567890abcdef
    export OPENMETER_URL=https://openmeter.cloud  # this is the default and you only need to set when self-hosting
    export NAMESPACE=default # this is the default, change to the namespace you want to watch
    export MONITOR_RATE_SECONDS=5 # this is the default, change to the rate you want to monitor pods (i.e. how often should we check for alive pods)
    ```

2. Run the binary

    ```bash
    ./openmeter-pod-runtime-watcher
    ```

## How it works

This application will query for pods in a namespace with the `k8s.openmeter.cloud/monitor` label set.
For each pod found, it will look up each label that starts with `k8s.openmeter.cloud/data-` and use the suffix as the key to send to OpenMeter.
Here's a full example of a pod and the data it will send to OpenMeter:

```yaml
# source: examples/k8s-manifests/pod.yaml
kind: Pod
apiVersion: v1
metadata:
  name: openmeter-pod-runtime-watcher-example
  labels:
    k8s.openmeter.cloud/monitor: kernel_runtime
    k8s.openmeter.cloud/subject: eli-test
    k8s.openmeter.cloud/data-file_id: 00703462-bb3e-4d99-966e-518fa207fcf8
    k8s.openmeter.cloud/data-kernel_session_id: 934f5a4b-8f59-44bd-a69c-2d39bda56aad
    k8s.openmeter.cloud/data-project_id: 6f0dff14-67d4-4cdf-bca4-b60019211e1a
    k8s.openmeter.cloud/data-space_id: 285698d3-23cc-4816-b891-d554351611d0
    k8s.openmeter.cloud/data-hardware_kind: cpu
    k8s.openmeter.cloud/data-hardware_size: small
spec:
  containers:
    - name: busybox
      image: busybox
      command: ['sh', '-c', 'echo The app is running! && sleep 3600']
```

This will map to the following cloudevents JSON request to OpenMeter:

```json
{
    "specversion": "1.0",
    "type": "kernel_runtime",
    "source": "kubernetes-api",
    "subject": "eli-test",
    "id": "9c6d076d-3c8f-456a-9df1-85e247e34151",
    "time": "2023-10-03T20:00:00Z",
    "datacontenttype": "application/json",
    "data": {
        "file_id": "00703462-bb3e-4d99-966e-518fa207fcf8",
        "kernel_session_id": "934f5a4b-8f59-44bd-a69c-2d39bda56aad",
        "project_id": "6f0dff14-67d4-4cdf-bca4-b60019211e1a",
        "space_id": "285698d3-23cc-4816-b891-d554351611d0",
        "hardware_kind": "cpu",
        "hardware_size": "small",
        "duration": "5"
    }
}
```

The rate at which events are sent for pods is configurable using the `MONITOR_RATE_SECONDS` environment variable.
Or, the `settings.monitor_rate_seconds` helm chart value.

The source can also be configured using the `SOURCE` environment variable or the `settings.source` helm chart value.

By default, we send events to the https://openmeter.cloud API, but if you self-host, this can be configured using the `OPENMETER_URL` environment variable or the `settings.openmeter_url` helm chart value.
