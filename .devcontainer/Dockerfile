# Do all the cargo install stuff
FROM rust:slim-bookworm AS builder

# Configure apt and install packages
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        libpq-dev \
        curl \
        wget \
        xz-utils \
        build-essential \
        libssl-dev \
        librust-openssl-dev \
        python3 python3-pip \
        pkg-config \
        unzip

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -y | sh

RUN curl https://github.com/watchexec/cargo-watch/releases/download/v8.5.2/cargo-watch-v8.5.2-aarch64-apple-darwin.tar.xz -L -o cargo-watch.tar.xz \
    && tar -xf cargo-watch.tar.xz \
    && mv cargo-watch-v8.5.2-aarch64-apple-darwin/cargo-watch /home

RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install cargo-release
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install sqlx-cli

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
# RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
# RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
# RUN cp cargo-binstall /usr/local/cargo/bin

# Install cargo-leptos
# RUN cargo binstall cargo-leptos -y

FROM rust:slim-bookworm

ARG DOCKER_COMPOSE_VERSION=2.23.0
ARG MOLD_VERSION=2.30.0
ARG ARCH=aarch64

# This Dockerfile adds a non-root 'vscode' user with sudo access. However, for Linux,
# this user's GID/UID must match your local user UID/GID to avoid permission issues
# with bind mounts. Update USER_UID / USER_GID if yours is not 1000. See
# https://aka.ms/vscode-remote/containers/non-root-user for details.
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Configure apt and install packages
RUN apt-get -y update \
    && apt-get install -y --no-install-recommends \
        git \
        curl \
        wget \
        ssh \
        sudo \
        jq \
        build-essential \
        protobuf-compiler \
        ca-certificates \
        software-properties-common \
        mariadb-client \
        gnupg-agent \
        gnupg \
        npm \
        nodejs \
        zsh \
        git \
        pkg-config \
        python3 python3-pip \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -r /var/cache/* /var/lib/apt/lists/*

RUN install -m 0755 -d /etc/apt/keyrings \
    && curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg \
    && chmod a+r /etc/apt/keyrings/docker.gpg \
    && curl -fsSL "https://download.docker.com/linux/debian/gpg" | apt-key add - \
    && echo \
        "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian \
        "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
        tee /etc/apt/sources.list.d/docker.list > /dev/null \
    && apt-get -y update \
    && apt-get -y --no-install-recommends install docker-ce docker-ce-cli containerd.io \
    && apt-get autoremove -y && apt-get clean -y

    # Create a non-root user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd -s /usr/bin/zsh --uid $USER_UID --gid $USER_GID -m $USERNAME -d /home/$USERNAME \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME\
    && chmod 0440 /etc/sudoers.d/$USERNAME \
    && usermod -aG docker $USERNAME \
    # Rust tools
    && rustup component add rustfmt clippy \
    # Add the musl toolchain
    && rustup target add x86_64-unknown-linux-musl \
    && rustup target add wasm32-unknown-unknown


    # Mold - Fast Rust Linker
RUN curl -OL https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/mold-$MOLD_VERSION-$ARCH-linux.tar.gz \
    && tar -xf mold-$MOLD_VERSION-$ARCH-linux.tar.gz \
    && mv ./mold-$MOLD_VERSION-$ARCH-linux/bin/mold /usr/bin/ \
    && mv ./mold-$MOLD_VERSION-$ARCH-linux/lib/mold/mold-wrapper.so /usr/bin/ \
    && rm mold-$MOLD_VERSION-$ARCH-linux.tar.gz \
    && rm -rf ./mold-$MOLD_VERSION-$ARCH-linux \
    && chmod +x /usr/bin/mold

RUN curl -L https://github.com/docker/compose/releases/download/v$DOCKER_COMPOSE_VERSION/docker-compose-linux-$ARCH -o /usr/local/bin/docker-compose \
    && chmod +x /usr/local/bin/docker-compose

# Just
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin \
    && mv ~/bin/just /usr/local/bin

# Docker debian
RUN curl https://raw.githubusercontent.com/microsoft/vscode-dev-containers/main/script-library/docker-debian.sh \
| bash -s -- true /var/run/docker-host.sock /var/run/docker.sock vscode false latest

# Cargo utils
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install cargo-release
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install sqlx-cli
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install cargo watch
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install cargo make
RUN CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse cargo install typeshare-cli

# Install cfssl
RUN curl -fsSL https://github.com/cloudflare/cfssl/releases/download/v1.6.4/cfssl_1.6.4_linux_arm64 -o /usr/local/bin/cfssl \
    && curl -fsSL https://github.com/cloudflare/cfssl/releases/download/v1.6.4/cfssljson_1.6.4_linux_arm64 -o /usr/local/bin/cfssljson \
    && chmod +x /usr/local/bin/cfssl /usr/local/bin/cfssljson

RUN mkdir -p /workspace && chown -R $USERNAME:$USERNAME /workspace

USER $USERNAME

# Copy the binaries we built in builder container

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

COPY --chown=$USERNAME --from=builder /home/cargo-watch $CARGO_HOME/bin
COPY --chown=$USERNAME ./zshrc /home/$USERNAME/.zshrc
