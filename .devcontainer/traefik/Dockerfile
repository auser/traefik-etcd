FROM docker.io/alpine:3.20 AS etcd

ARG ETCD_VER
ENV ETCD_VER=3.5.15

RUN apk add --update ca-certificates openssl tar curl
RUN arch=$(arch | sed s/aarch64/arm64/ | sed s/x86_64/amd64/) && \
    curl -L https://github.com/etcd-io/etcd/releases/download/v${ETCD_VER}/etcd-v${ETCD_VER}-linux-${arch}.tar.gz -o /tmp/etcd.tar.gz && \
    mkdir -p /tmp/etcd-download && \
    tar xzvf /tmp/etcd.tar.gz -C /tmp/etcd-download --strip-components=1 --no-same-owner && \
    rm -Rf /tmp/etcd.tar.gz && \
    mv /tmp/etcd-download/etcd* /bin/ && \
    apk del --purge tar openssl && \
    rm -Rf /tmp/etcd-download /var/cache/apk/*

FROM docker.io/traefik:v3.2.1

# Install envsubst
RUN apk update && apk add --no-cache gettext shadow

RUN apk add --no-cache libcap && \
    setcap 'cap_net_bind_service=+ep' /usr/local/bin/traefik

# Copy basicAuth - insecure
COPY ./.devcontainer/traefik/.htpasswd /etc/traefik/.htpasswd

COPY ./.devcontainer/traefik/traefik.yml /etc/traefik/traefik.yaml
COPY ./.devcontainer/traefik/dynamic.yml /etc/traefik/dynamic_conf.yaml

# Should we do it this way? 
# Nope, but good enough for now

# Server certificates
COPY ./config/tls/etcd.pem /etc/traefik/server.pem
COPY ./config/tls/etcd-key.pem /etc/traefik/server-key.pem
COPY ./config/tls/ca.pem /etc/traefik/ca.pem

COPY ./config/tls/etcd-peer.pem /etc/traefik/peer.pem
COPY ./config/tls/etcd-peer-key.pem /etc/traefik/peer-key.pem

COPY ./config/tls/traefik-etcd-client.pem /etc/traefik/etcd-client.pem
COPY ./config/tls/traefik-etcd-client-key.pem /etc/traefik/etcd-client-key.pem

COPY --from=etcd /bin/etcd /bin/etcd
COPY --from=etcd /bin/etcdctl /bin/etcdctl

COPY ./.devcontainer/traefik/traefik-entrypoint.sh /docker-entrypoint.sh

VOLUME      /data
VOLUME      /logs

EXPOSE      2379 2380 4001 7001

RUN groupadd -r traefik && \
useradd -ms /bin/sh -d /home/traefik -g traefik traefik

RUN chmod +x /docker-entrypoint.sh && \
    chown traefik:traefik /docker-entrypoint.sh && \
    mkdir -p /etc/traefik && mkdir -p /logs

RUN chmod 644 /etc/traefik/*.pem && \
    chmod 600 /etc/traefik/*-key.pem && \
    chown traefik:traefik -R /etc/traefik && \
    chown traefik:traefik -R /logs

USER traefik

ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["traefik"]

