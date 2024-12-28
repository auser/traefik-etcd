FROM --platform=${PLATFORM} docker.io/bitnami/etcd:3.5
ARG PLATFORM=${PLATFORM:-linux/amd64}

ARG CERT_DIR=${CERT_DIR:-./config/tls}

USER root

COPY $CERT_DIR/etcd.pem /etc/etcd/tls/server.pem
COPY $CERT_DIR/etcd-key.pem /etc/etcd/tls/server-key.pem
COPY $CERT_DIR/ca.pem /etc/etcd/tls/ca.pem

COPY $CERT_DIR/etcd-peer.pem /etc/etcd/tls/peer.pem
COPY $CERT_DIR/etcd-peer-key.pem /etc/etcd/tls/peer-key.pem

COPY ./.devcontainer/etcd/etcd.yml /etc/etcd/config.yml
COPY ./.devcontainer/etcd/setup-etcd.sh /usr/local/bin/setup_etcd.sh
COPY ./.devcontainer/etcd/etcd-entrypoint.sh /usr/local/bin/etcd-entrypoint.sh
COPY ./.devcontainer/etcd/etcd-healthcheck.sh /etcd-healthcheck.sh
COPY ./.devcontainer/etcd/bashrc /root/.bashrc

RUN groupadd -r etcd && \
    useradd -ms /bin/bash -d /home/etcduser -g etcd etcduser

RUN chmod +x /usr/local/bin/setup_etcd.sh /usr/local/bin/etcd-entrypoint.sh && \
    mkdir -p /etcd/data && \
    chown -R etcduser:etcd /etcd && \
    chmod -R 0700 /etcd && \
    chmod 0700 /etcd/data && \
    chmod 0755 /root/.bashrc

# Honestly, I don't know why this is needed, but it is
RUN mkdir -p /opt/bitnami/etcd/data && \
    chown -R etcduser:etcd /opt/bitnami && \
    chmod -R 744 /opt/bitnami

RUN chmod 644 /etc/etcd/tls/*.pem && \
    chmod 600 /etc/etcd/tls/*-key.pem && \
    chown etcduser:etcd -R /etc/etcd && \
    chown etcduser:etcd -R /usr/local/bin/etcd-entrypoint.sh && \
    chmod 755 /usr/local/bin/etcd-entrypoint.sh

USER etcduser

EXPOSE 2379 2380

ENTRYPOINT ["/usr/local/bin/etcd-entrypoint.sh"]
CMD ["/opt/bitnami/etcd/bin/etcd", "--config-file", "/etc/etcd/config.yml"]