#!/usr/bin/env bash

set -e

DIR_PATH=$(realpath $(dirname "$0"))



declare -A COLORS=(
    [Color_Off]='\033[0m'
    [Black]='\033[0;30m'
    [Red]='\033[0;31m'
    [Green]='\033[0;32m'
    [Yellow]='\033[0;33m'
    [Blue]='\033[0;34m'
    [Purple]='\033[0;35m'
    [Cyan]='\033[0;36m'
    [White]='\033[0;37m'
)

INPUT_FILE="${DIR_PATH}/../config/output.txt"
OUTPUT_FILE="${DIR_PATH}/reset.sh"  

ecd="/opt/bitnami/etcd/bin/etcdctl --endpoints=https://localhost:2379 --cacert=/etc/etcd/ca.pem --cert=/etc/etcd/server.pem --key=/etc/etcd/server-key.pem"

revert_all() {
    if [ ! -f "$INPUT_FILE" ]; then
        echo -e "${COLORS[Red]}Error: Input file $INPUT_FILE does not exist${COLORS[Color_Off]}"
        exit 1
    fi
    
    echo -e "#!/usr/bin/env bash\n\n" > "$OUTPUT_FILE"
    echo -e "export etcdctl_API=\${etcdctl_API:-3}\n" >> "$OUTPUT_FILE"
    echo -e "$ecd del --prefix traefik\n" >> "$OUTPUT_FILE"
    echo -e "$ecd put traefik true\n" >> "$OUTPUT_FILE"

    while IFS= read -r line; do
        echo -e "$ecd put $line" >> "$OUTPUT_FILE"
    done < "$INPUT_FILE"

    echo "All done!"
}

parse_opts() {
  local opt
  while getopts "vo:i:" opt; do
    case ${opt} in
    v)
      VERBOSE=true
      ;;
    o)
      OUTPUT_FILE=$OPTARG
      ;;
    i)
      INPUT_FILE=$OPTARG
      
      ;;
    \?)
      echo "Invalid option: $OPTARG" 1>&2
      exit 1
      ;;
    esac
  done
}

help() {
  echo -e "${COLORS[Blue]}Usage: $(basename "$0") [options] <command>
Options:
  -v  Verbose mode
  -i  Input file
  -o  Output file

Commands:
  ${COLORS[Green]}all${COLORS[Color_Off]}                            Revert all etcd entries
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
  all) revert_all ;;
  *) help ;;
  esac
}

main "$@"
