#!/bin/bash

set -e

DIR_PATH=$(realpath $(dirname "$0"))
source $DIR_PATH/colors.sh

CERT_BASE_DIR=${CERT_BASE_DIR:-"./config/tls"}

PROFILE=${PROFILE:-"server"}
NAME=${NAME:-"traefik"}
COMMON_NAME=${COMMON_NAME:-"traefik"}
HOSTS=${HOSTS:-"localhost,127.0.0.1,traefik,ari.io,*.ari.io"}
OUTPUT_DIR=${OUTPUT_DIR:-"./config/tls"}
BASE_DIR=${BASE_DIR:-"./config/tls"}
DOMAIN=${DOMAIN:-"ari.io"}

CFSSL=$(which cfssl)
CFSSLJSON=$(which cfssljson)

generate_ca() {
    # Generate CA if it doesn't exist
    if [ ! -f "${BASE_DIR}/ca.pem" ]; then
        cfssl gencert -initca "${BASE_DIR}/ca-csr.json" | cfssljson -bare "${BASE_DIR}/ca"
    fi
}

# ./scripts/gen-certs.sh -p server -c etcd -h localhost,127.0.0.1,etcd -o ./proxy/cert/etcd
generate_cert() {
    echo "Generating certificate for ${NAME} with common name ${COMMON_NAME} and hosts ${HOSTS}"

    JSON_HOSTS=$(echo "$HOSTS" | jq -R 'split(",")')
    echo $JSON_HOSTS

    mkdir -p "${OUTPUT_DIR}"
    jq --arg cn "$COMMON_NAME" --argjson hosts "$JSON_HOSTS" '.CN = $cn | .hosts = $hosts' "${BASE_DIR}/csr-template.json" > "${OUTPUT_DIR}/${NAME}-csr.json"


    # sed "s/COMMON_NAME/$COMMON_NAME/g; s/HOSTS/$JSON_HOSTS/g" "${BASE_DIR}/csr-template.json" > "${OUTPUT_DIR}/${NAME}-csr.json"
    # Generate the certificate

    cfssl gencert \
        -ca="${BASE_DIR}/ca.pem" \
        -ca-key="${BASE_DIR}/ca-key.pem" \
        -config="${BASE_DIR}/ca-config.json" \
        -profile=$PROFILE \
        "${OUTPUT_DIR}/${NAME}-csr.json" | cfssljson -bare "${OUTPUT_DIR}/${NAME}"

    # Clean up the temporary CSR file
    rm "${OUTPUT_DIR}/${NAME}-csr.json"
}


# # Generate etcd server certificate
# generate_cert "server" "etcd" "localhost,127.0.0.1,etcd" "etcd" "${CERT_BASE_DIR}/etcd"

# # Generate Traefik client certificate for etcd communication
# generate_cert "etcd-client" "traefik-etcd-client" "" "etcd" "${CERT_BASE_DIR}/traefik"

# # Generate Traefik server certificate
# generate_cert "server" "traefik" "localhost,127.0.0.1,traefik,your-domain.com,*.your-domain.com" "traefik" "${CERT_BASE_DIR}/traefik"

# echo "Certificates generated in ${CERT_BASE_DIR}"

function generate_etcd() {
    echo -e "${Colors[Yellow]}Generating etcd peer certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p peer \
        -c etcd-cluster \
        -h "localhost,127.0.0.1,0.0.0.0,etcd,traefik,$DOMAIN,*.$DOMAIN" \
        -n etcd-peer \
        -o $CERT_BASE_DIR \
        gen

    echo -e "${Colors[Yellow]}Generating etcd server certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p server \
        -c etcd-cluster \
        -h "localhost,127.0.0.1,0.0.0.0,etcd,traefik,$DOMAIN,*.$DOMAIN" \
        -n etcd \
        -o $CERT_BASE_DIR \
        gen
}

function generate_traefik() {
    echo -e "${Colors[Yellow]}Generating traefik server certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p server \
        -c traefik \
        -h "localhost,127.0.0.1,traefik,$DOMAIN,*.$DOMAIN" \
        -n traefik-server \
        -o $CERT_BASE_DIR \
        gen
}

function generate_asterisk() {
    echo -e "${Colors[Yellow]}Generating asterisk certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p server \
        -c asterisk \
        -h "localhost,127.0.0.1,traefik,asterisk,$DOMAIN,*.$DOMAIN" \
        -n asterisk \
        -o $CERT_BASE_DIR \
        gen
}

function generate_herringbank() {
    echo -e "${Colors[Yellow]}Generating herringbank certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p server \
        -c wildcard_herringbank \
        -h "localhost,127.0.0.1,traefik,herringbank,$DOMAIN,*.$DOMAIN" \
        -n wildcard_herringbank \
        -o $CERT_BASE_DIR \
        gen
}

function generate_etcd_traefik_communication() {
    echo -e "${Colors[Yellow]}Generating etcd traefik communication certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p client \
        -c traefik-etcd-client \
        -h "etcd,traefik,$DOMAIN,*.$DOMAIN" \
        -n traefik-etcd-client \
        -o $CERT_BASE_DIR \
        gen
}

function generate_prometheus() {
    echo -e "${Colors[Yellow]}Generating prometheus certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p server \
        -c prometheus \
        -h "prometheus,$DOMAIN,*.$DOMAIN" \
        -n prometheus \
        -o $CERT_BASE_DIR \
        gen
}

function generate_grafana() {
    echo -e "${Colors[Yellow]}Generating grafana certificate${Colors[Color_Off]}"
    ./scripts/gen-certs.sh \
        -p server \
        -c grafana \
        -h "grafana,$DOMAIN,*.$DOMAIN" \
        -n grafana \
        -o $CERT_BASE_DIR \
        gen
}

function generate_all() {
  mkdir -p "${BASE_DIR}"
  # generate_ca
  generate_etcd
  generate_traefik
  generate_etcd_traefik_communication
  generate_prometheus
  generate_grafana
}

function generate_asterisk_herringbank() {
  generate_asterisk
  generate_herringbank
}

parse_opts() {
  local opt
  while getopts "p:c:n:h:o:d:" opt; do
    case ${opt} in
    p) PROFILE=$OPTARG ;;
    c) COMMON_NAME=$OPTARG ;;
    h) HOSTS=$OPTARG ;;
    n) NAME=$OPTARG ;;
    o) OUTPUT_DIR=$OPTARG ;;
    d) DOMAIN=$OPTARG ;;
    \?)
      echo "Invalid option: $OPTARG" 1>&2
      exit 1
      ;;
    esac
  done
}

help() {
  echo -e "${Colors[Blue]}Usage: $(basename "$0") [options] <command>
Options:
  -p  Profile
  -c  Common Name
  -n  Name
  -h  Hosts
  -o  Output directory
  -d  Domain
  -v  Verbose mode

Commands:
  ${Colors[Green]}all${Colors[Color_Off]}                            Generate all certificates
  ${Colors[Green]}ca${Colors[Color_Off]}                             Generate CA
  ${Colors[Green]}gen${Colors[Color_Off]}                            Generate certificate
  ${Colors[Green]}gen_etcd${Colors[Color_Off]}                       Generate etcd certificate
  ${Colors[Green]}gen_traefik${Colors[Color_Off]}                    Generate traefik certificate
  ${Colors[Green]}gen_etcd_traefik_communication${Colors[Color_Off]} Generate etcd traefik communication certificate
  ${Colors[Green]}gen_prometheus${Colors[Color_Off]}                 Generate prometheus certificate
  ${Colors[Green]}gen_grafana${Colors[Color_Off]}                    Generate grafana certificate
  ${Colors[Green]}gen_asterisk_herringbank${Colors[Color_Off]}       Generate asterisk and herringbank certificates
"
  exit 1
}

main() {
  parse_opts "$@"
  shift $((OPTIND - 1))
  if [ $# -eq 0 ]; then
    help
  fi
  case "$1" in
  all) generate_all ;;
  ca) generate_ca ;;
  gen) generate_cert ;;
  gen_etcd) generate_etcd ;;
  gen_traefik) generate_traefik ;;
  gen_etcd_traefik_communication) generate_etcd_traefik_communication ;;
  gen_prometheus) generate_prometheus ;;
  gen_grafana) generate_grafana ;;
  gen_asterisk_herringbank) generate_asterisk_herringbank ;;
  ca) gen_ca ;;
  *) help ;;
  esac
}

main "$@"
