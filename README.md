# Rust Microservice Chassis

[TOC]

## Getting started

A Linux (WSL) environment is assumed and is recommended.

The chassis uses the Ericsson Rust Library (ERS) which is in the same workspace
allowing easy development of multiple packages at the same time.

### Building Dev Docker Image

```sh
bob create-build-image
```

### Building the Microservice

```sh
bob build
```

### Testing the Microservice

```sh
bob test
```

### Building the Microservice Docker Image

```sh
bob create-image
```

### Starting the Microservice Docker Image

```sh
bob run-locally
```

### Building and Running at Once

```sh
bob build create-image run-locally
```

## Building and running in native environment

```sh
RUST_LOG=debug cargo run
```

An example command to run the Kafka client with debug logging. Note, that by
default only error logs are logged.
Arguments can be passed from Cargo to executable via double dashes (`--`).

```sh
RUST_LOG=debug cargo run --bin example_kafka -- --num-records 1
```

### Dependencies

Confluent platform can be deployed locally. It can be downloaded and extracted
with the following command.

```sh
cd ~/.local
curl -O https://packages.confluent.io/archive/7.5/confluent-7.5.3.tar.gz
tar -xf confluent-7.5.3.tar.gz

LOG_DIR=/tmp/zoo-logs ~/.local/confluent-7.5.3/bin/zookeeper-server-start -daemon ~/.local/confluent-7.5.3/etc/kafka/zookeeper.properties
LOG_DIR=/tmp/kafka-logs ~/.local/confluent-7.5.3/bin/kafka-server-start -daemon ~/.local/confluent-7.5.3/etc/kafka/server.properties

```

## Project structure

```
├── Cargo.toml                  Project wide Cargo containing workspace (library and application code)
├── Dockerfile                  Microservice runtime environment
├── buildenv.Dockerfile         Build environment (Used by `bob build`)
├── doc                         General documentation (Developer Manual, guidelines, tips and tricks, etc...)
├── ers                         Ericsson Rust Library
├── ms                          Application code (the microservice)
├── ruleset2.0.yaml             CI pipeline
└── settings.yaml               Configuration of the microservice
```
