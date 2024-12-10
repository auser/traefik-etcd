#!/bin/sh

set -ex

export ETCDCTL_API=3
ETCDCTL=/opt/bitnami/etcd/bin/etcdctl

ETCDCTL_FLAGS="\
    --endpoints=https://etcd:2379 \
    --cacert=/etc/etcd/tls/ca.pem \
    --cert=/etc/etcd/tls/server.pem \
    --key=/etc/etcd/tls/server-key.pem"

# Check if etcd is healthy
${ETCDCTL} ${ETCDCTL_FLAGS} get /traefik/dummy > /dev/null 2>&1