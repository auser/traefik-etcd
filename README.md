# Traefik Config Generator with etcd

This is a tool to generate Traefik configuration for a given set of hosts.

## Usage

```bash
cargo run --bin traefik-config-generator --config config/config.yml
```

## Installation

Head to [https://auser.github.io/traefikctl/](https://auser.github.io/traefikctl/) for installation instructions.

## Configuration

The configuration is done in the `config/config.yml` file.

### Hosts

Each host has a domain, a list of paths, and a list of deployments.

#### Paths

Each path has a path, a list of deployments, a list of middlewares, and a boolean to strip the prefix. The deployments are keyed by the deployment name, which is used to determine which router to use.

#### Deployments

Each deployment has an ip, a port, a weight, and a boolean to determine if the cookie should be passed through.

It can also have a list of weights for each deployment.

```yaml
etcd:
  endpoints: ["https://0.0.0.0:2379"]
  timeout: 2000
  keep_alive: 300
  tls:
    cert: "./config/tls/etcd-peer.pem"
    key: "./config/tls/etcd-peer-key.pem"
    ca: "./config/tls/ca.pem"
    domain: herringbank.com

middlewares:
  enable-headers:
    headers:
      custom_request_headers:
        X-Forwarded-Proto: "https"
        X-Forwarded-Port: "443"
        Location: ""
      custom_response_headers:
        Location: ""
      access_control_allow_methods:
        - "GET"
      access_control_allow_headers:
        - "Content-Type"
      access_control_expose_headers:
        - Location
      add_vary_header: true

hosts:
  - domain: "example.com"
    www_redirect: true
    paths:
      - path: "/test"
        deployments:
          blue:
            ip: 10.0.0.1
            port: 8080
            weight: 50
          green:
            ip: 10.0.0.2
            port: 8080
            weight: 50
        middlewares:  
          - enable-headers
          - forward-server

    # Root path (catch-all)
    deployments:
      blue:
        ip: 10.0.0.1
        port: 8080
        weight: 100
```

## Running over an ssh tunnel

```
ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -L 2379:0.0.0.0:2379 alerner@proxy
```