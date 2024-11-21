#!/bin/bash

export ETCDCTL_API=3
ETCDCTL=/opt/bitnami/etcd/bin/etcdctl

ETCDCTL_FLAGS="\
    --endpoints=https://localhost:2379 \
    --cacert=/etc/etcd/ca.pem \
    --cert=/etc/etcd/server.pem \
    --key=/etc/etcd/server-key.pem"

# Wait for etcd to be ready
echo -e "\n\nWaiting for etcd..."
until $ETCDCTL $ETCDCTL_FLAGS get foo > /dev/null 2>&1; do
    echo "Waiting for etcd..."
    sleep 2
done
echo "etcd is ready"

$ETCDCTL $ETCDCTL_FLAGS put traefik -- true

$ETCDCTL $ETCDCTL_FLAGS get traefik

# export PROXY_DOMAIN=herringbank.fp
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/rule "Host(\`demo.${PROXY_DOMAIN}\`)"
# # $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/entrypoints/0 "websecure"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/tls/domains/0/main "demo.${PROXY_DOMAIN}"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/tls/domains/0/sans/0 "*.${PROXY_DOMAIN}"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/tls/domains/1/main "demo2.${PROXY_DOMAIN}"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/tls/domains/1/sans/0 "*.${PROXY_DOMAIN}"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/routers/demo-router/service/loadbalancer/servers/0/url "https://demo.${PROXY_DOMAIN}"
# # Add backends
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/app-v1/loadBalancer/servers/0/url "http://app-v1"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/app-v2/loadBalancer/servers/0/url "http://app-v2"

# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/weighted-app/weighted/services/0/name "app-v1"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/weighted-app/weighted/services/0/weight "5"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/weighted-app/weighted/services/1/name "app-v2"
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/weighted-app/weighted/services/1/weight "5"


# Adding sticky sessions
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/demo-service/loadbalancer/sticky true
# $ETCDCTL $ETCDCTL_FLAGS put traefik/http/services/demo-service/loadbalancer/sticky/cookie/name "demo"

echo "Etcd populated successfully"