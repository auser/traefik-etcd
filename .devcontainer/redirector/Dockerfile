FROM auser/redirector:latest
ARG PLATFORM=linux/amd64
ENV PLATFORM=${PLATFORM:-linux/amd64}

USER root

ARG USERNAME=vscode
ARG USER_UID=1001
ARG USER_GID=$USER_UID

ENV PLATFORM=${PLATFORM:-linux/amd64}
ENV USERNAME=${USERNAME:-vscode}
ENV USER_UID=${USER_UID:-1001}
ENV USER_GID=${USER_GID:-1001}

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get -y update \
    && apt-get install -y --no-install-recommends \
        curl \
        sudo \
        jq \
        build-essential \
        zsh \
        git \
        pkg-config \
        python3 python3-pip \
        vim \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -r /var/cache/* /var/lib/apt/lists/*

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd -s /usr/bin/zsh --uid $USER_UID --gid $USER_GID -m $USERNAME -d /home/$USERNAME \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME\
    && chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME