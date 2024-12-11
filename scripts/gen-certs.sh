#!/usr/bin/env bash



DIR_PATH=$(realpath $(dirname "$0"))
source $DIR_PATH/colors.sh

CERT_BASE_DIR=${CERT_BASE_DIR:-"./config/tls"}

PROFILE=${PROFILE:-"server"}
NAME=${NAME:-"traefik"}
COMMON_NAME=${COMMON_NAME:-"traefik"}
HOSTS=${HOSTS:-"localhost,traefik"}
OUTPUT_DIR=${OUTPUT_DIR:-"./config/tls"}
BASE_DIR=${BASE_DIR:-"./config/tls"}
DOMAIN=${DOMAIN:-"ari.io"}

# Check if cfssl is installed
if ! command -v cfssl &> /dev/null; then
    echo -e "${Red}Error: cfssl is not installed ${COLOR_OFF}"
    echo "Please install cfssl first:"
    echo "  go install github.com/cloudflare/cfssl/cmd/cfssl@latest"
    exit 1
fi

# Check if cfssljson is installed 
if ! command -v cfssljson &> /dev/null; then
    echo -e "${Red}Error: cfssljson is not installed ${COLOR_OFF}"
    echo "Please install cfssljson first:"
    echo "  go install github.com/cloudflare/cfssl/cmd/cfssljson@latest" 
    exit 1
fi

# Set paths to binaries
CFSSL=$(command -v cfssl)
CFSSLJSON=$(command -v cfssljson)

ensure_directory_exists() {
    mkdir -p "${OUTPUT_DIR}"
}

generate_ca() {
    ensure_directory_exists
    # Generate CA if it doesn't exist
    if [ ! -f "${BASE_DIR}/ca.pem" ]; then
        cfssl gencert -initca "${BASE_DIR}/ca-csr.json" | cfssljson -bare "${BASE_DIR}/ca"
    fi
}

# ./scripts/gen-certs.sh -p server -c etcd -h localhost,127.0.0.1,etcd -o ./proxy/cert/etcd
generate_cert() {
    ensure_directory_exists
    echo -e "${YELLOW}Generating certificate for ${NAME} with common name ${COMMON_NAME} and hosts ${HOSTS} ${COLOR_OFF}"

    JSON_HOSTS=$(echo "$HOSTS" | jq -R 'split(",")')

    echo "${OUTPUT_DIR}/${NAME}-csr.json"

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
    ensure_directory_exists
    echo -e "${YELLOW}Generating etcd peer certificate $COLOR_OFF"
    ./scripts/gen-certs.sh \
        -p peer \
        -c etcd-cluster \
        -h "localhost,127.0.0.1,0.0.0.0,etcd,traefik,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n etcd-peer \
        -o $CERT_BASE_DIR \
        gen

    echo -e "${YELLOW}Generating etcd server certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p server \
        -c etcd-cluster \
        -h "localhost,127.0.0.1,0.0.0.0,etcd,traefik,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n etcd \
        -o $CERT_BASE_DIR \
        gen
}

function generate_traefik() {
      ensure_directory_exists
    echo -e "${YELLOW}Generating traefik server certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p server \
        -c traefik \
        -h "localhost,127.0.0.1,traefik,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n traefik \
        -o $CERT_BASE_DIR \
        gen
}

function generate_asterisk() {
    ensure_directory_exists
    echo -e "${YELLOW}Generating asterisk certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p server \
        -c asterisk \
        -h "localhost,127.0.0.1,traefik,asterisk,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n asterisk \
        -o $CERT_BASE_DIR \
        gen
}

function generate_herringbank() {
    ensure_directory_exists
    echo -e "${YELLOW}Generating herringbank certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p server \
        -c wildcard_herringbank \
        -h "localhost,127.0.0.1,traefik,herringbank,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n wildcard_herringbank \
        -o $CERT_BASE_DIR \
        gen
}

function generate_etcd_traefik_communication() {
    ensure_directory_exists
    echo -e "${YELLOW}Generating etcd traefik communication certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p client \
        -c traefik-etcd-client \
        -h "etcd,traefik,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n traefik-etcd-client \
        -o $CERT_BASE_DIR \
        gen
}

function generate_prometheus() {
    ensure_directory_exists
    echo -e "${YELLOW}Generating prometheus certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p server \
        -c prometheus \
        -h "prometheus,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n prometheus \
        -o $CERT_BASE_DIR \
        gen
}

function generate_grafana() {
    ensure_directory_exists
    echo -e "${YELLOW}Generating grafana certificate ${COLOR_OFF}"
    ./scripts/gen-certs.sh \
        -p server \
        -c grafana \
        -h "grafana,$DOMAIN,*.$DOMAIN,$HOSTS" \
        -n grafana \
        -o $CERT_BASE_DIR \
        gen
}

function generate_all() {
  ensure_directory_exists
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
    h) HOSTS="$HOSTS,$OPTARG" ;;
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
  echo -e "${BLUE}Usage: $(basename "$0") [options <command>
Options:
  -p  Profile
  -c  Common Name
  -n  Name
  -h  Hosts
  -o  Output directory
  -d  Domain
  -v  Verbose mode

Commands:
  ${GREEN}all  ${COLOR_OFF}                             Generate all certificates
  ${GREEN}ca  ${COLOR_OFF}                              Generate CA
  ${GREEN}gen  ${COLOR_OFF}                             Generate certificate
  ${GREEN}gen_etcd  ${COLOR_OFF}                        Generate etcd certificate
  ${GREEN}gen_traefik  ${COLOR_OFF}                     Generate traefik certificate
  ${GREEN}gen_etcd_traefik_communication  ${COLOR_OFF}  Generate etcd traefik communication certificate
  ${GREEN}gen_prometheus ${COLOR_OFF}                   Generate prometheus certificate
  ${GREEN}gen_grafana ${COLOR_OFF}                      Generate grafana certificate
  ${GREEN}gen_asterisk_herringbank  ${COLOR_OFF}        Generate asterisk and herringbank certificates
"
  exit 1
}


main() {
  parse_opts "$@"
  shift $((OPTIND - 1))
  if [ $# -eq 0 ]; then
    help
  fi

  CERT_BASE_DIR=${OUTPUT_DIR:-"./config/tls"}
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
