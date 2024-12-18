# Traefik Dynamic Configuration Manager

A Rust library for managing Traefik dynamic configuration through etcd.

## Installation

Head to [https://auser.github.io/traefikctl/](https://auser.github.io/traefikctl/) for installation instructions.

## Configuration

The configuration is done in the `config/config.yml` file. You can also pass in a partial etcd config via the cli to override the default config.

```
traefikctl get -f ./config/config-devcontainer.yml --etcd-config='{"endpoints": ["https://0.0.0.0:2379"], "tls": {"cert": "./config/tls/etcd-peer.pem", "key": "./config/tls/etcd-peer-key.pem", "ca": "./config/tls/ca.pem", "domain": "etcd"}}'
```

The configuration file actually contains a script using the handlebars syntax ([tera](https://docs.rs/tera/latest/tera/)) that is used to generate the configuration file, so you can use the `render` command to see what exactly is being generated before being used.

```
traefikctl render -f ./config/config-devcontainer.yml
```

There is a helper to use the environment variables to save typing `-f {config_file}`. Use `.envrc` to load the environment variables:

```
source .envrc
# Or 
direnv allow
```

## Getting Started

There are a few scripts to help you get started.

```
# Build the docker image (to contain common dependencies)
./scripts/devex.sh build

# Start the docker container
./scripts/devex.sh start

# Exec into the container
./scripts/devex.sh exec
```

### Reset the container

Occasionally you may need to reset the container. This will remove the container, rebuild the image, and start a new one. This uses the `devcontainer` command. If you do not have the `devcontainer` command, you can open vscode and run the `install devcontainer` command using the command palette.

```
./scripts/devex.sh reset
```

## Architecture

This project is built with a few goals in mind:

- Keep the configuration of _traefik_ as simple as possible using etcd and a simple configuration format.

Traefik is configured using a simple configuration format that is easy to understand and modify. The configuration is stored in etcd, and is automatically synced to the container. in addition, there is a frontend web app you can use to manage the configuration (more on that later).

`traefikctl` is a cli tool that is used to manage the configuration. It can be executed using the `traefikctl` command or through source using `cargo run`. To see all of the commands, you can run `cargo run -- --help` or `traefikctl --help`.

To get started, you can run `traefikctl get` to see the current configuration, but you can generate your own config file using the `traefikctl generate` command.

```
traefikctl generate -o ./config/generated.yml
```

In the case of using the `devcontainer` command, you'll need to generate ssl certificates for the etcd server. You can use the `./scripts/gen-certs.sh` script to generate the certificates. Run the command to see all of the options.

After you generate the certificates, you'll need to load them into the container.

The helpful script `./scripts/devex.sh` can be used to launch the devcontainer, load the certificates, run etcd, traefik, and (eventually) the frontend.

### Hosts

Each host has a domain, a list of paths, and a list of deployments.

#### Paths

Each path has a path, a list of deployments, a list of middlewares, and a boolean to strip the prefix. The deployments are keyed by the deployment name, which is used to determine which router to use.

#### Deployments

Each deployment has an ip, a port, a weight, and a boolean to determine if the cookie should be passed through.

It can also have a list of weights for each deployment.

**The root of the project are deployments.** Every deployment will create a router in Traefik as well as a service. You can configure the deployment to handle [Traefik](https://doc.traefik.io/traefik) routes as well as `Kubernetes` routes. 

## Features

- Strongly typed configuration using Rust structs that are automatically exported to TypeScript
- Support for blue/green deployments with weighted load balancing
- Middleware configuration for headers, TLS, and more
- Host and path-based routing
- Integration with etcd key-value store

## Configuration Example

The configuration is defined in YAML format. Here's an example:

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

### Connecting to etcd

You can connect to etcd using a TLS certificate, or over an ssh tunnel. The `endpoints` field in the config file should be a list of all the etcd endpoints you want to connect to. If you are connecting over tls, you will need to provide the cert, key, and ca files. as the `tls` field.

### Middleware Configuration

Middlewares are configured in the `middlewares` section. Each middleware has a name, and a set of options that are specific to the middleware. The middleware name is the name of the middleware in Traefik. The middleware name is used to apply the middleware to a path.

### Host Configuration

Hosts are configured in the `hosts` section. Each host has a domain, a list of paths, and a list of deployments. The domain is used to determine which router to use in Traefik. The paths are used to determine which deployments to use for the path.

Without `paths`, you can configure the host to catch all paths. with a root `deployments` section. If you want to configure a specific path, you can do so with the `paths` section.

### Keys in deployments

- `ip` - The ip address of the deployment
- `port` - The port of the deployment
- `weight` - The weight of the deployment
- `protocol` - The protocol to use to connect to the deployment. Defaults to `http` but you can set it to `tls`.

## Running over an ssh tunnel

```
ssh -o UserKnownHostsFile=/dev/null -o StrictHostKeyChecking=no -L 2379:0.0.0.0:2379 alerner@proxy
```

## Frontend

The frontend is a simple web app that is used to manage the configuration. It is built with [Svelte](https://svelte.dev/) and [Skeleton](http://getskeleton.com/). 

It is not built with any frameworks in mind, so it could be hosted on any static file server.

There is a helpful command to start the frontend  -- `cargo make dev`.

> If you get an error on a mac, you'll need to reinstall the `cargo-make` crate.
> Cannot run macOS (Mach-O) executable in Docker: Exec format error
>
> ```
> cargo install cargo-make --force
> ```

## Dev notes

Check the etcd container for keys:

```
# Find the etcd container ID
docker ps --format '{{.ID}} {{.Image}} {{.Names}}' | awk '($2 ~ /docker.io\/bitnami\/etcd/ || $3 ~ /etcd$/) {print $1}'

# Or as a one-liner:
ETCD_ID=$(docker ps --format '{{.ID}} {{.Image}} {{.Names}}' | awk '($3 ~ /etcd/) {print $1}')

# Then use it like:
docker exec -it $ETCD_ID etcdctl get /traefik/config --prefix
# Or as a one-liner:
docker exec -it $(docker ps --format '{{.ID}} {{.Image}} {{.Names}}' | awk '($3 ~ /etcd/) {print $1}') bash

export ecd="/opt/bitnami/etcd/bin/etcdctl --endpoints=https://localhost:2379 --cacert=/etc/etcd/tls/ca.pem --cert=/etc/etcd/tls/server.pem --key=/etc/etcd/tls/server-key.pem"
```
