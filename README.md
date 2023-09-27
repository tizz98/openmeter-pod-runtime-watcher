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
