#!/usr/bin/env bash

DIR_PATH=$(realpath $(dirname "$0"))
# source $DIR_PATH/colors.sh

IMAGE_NAME="auser/traefikctl"
IMAGE_TAG="latest"
CONTAINER_NAME="traefikctl_devcontainer-development"
FORCE_REBUILD_IMAGE="true"
FORCE_RESET_CONTAINER="true"
DOCKER_DIR=".devcontainer"
RUN_PRIVILEGED="false"
VERBOSE="false"
ADDITIONAL_ARGS=""
declare -a MOUNTS=("$(pwd):/workspace")

# Check if devcontainer binary exists
DEVCONTAINER_BIN=$(which devcontainer 2>/dev/null)
if [[ -z "$DEVCONTAINER_BIN" ]]; then
    printf "${RED}Error: devcontainer CLI not found. Please install it first.${COLOR_OFF}\n"
    exit 1
fi


# docker_service_address=$(docker network inspect kind -f "{{(index .IPAM.Config 1).Subnet}}" | cut -d '.' -f1,2,3)
# my_ip=$(ipconfig getifaddr en0)
# api_server_address="${my_ip}"

docker_instance() {
    docker ps --format '{{.ID}} {{.Names}}' | awk '($2 ~ /'$CONTAINER_NAME'/) {print $1}'                                                          
}

build_image() {
    local image_id=$(docker images --filter=reference="$IMAGE_NAME" --format "{{.ID}}")
    if [[ "$FORCE_REBUILD_IMAGE" == "true" && -n "$image_id" ]]; then
        docker rmi "$image_id"
    fi
    local cmd=(docker build) 
    cmd+=(-t "$IMAGE_NAME:$IMAGE_TAG")
    cmd+=(-f $DOCKER_DIR/Dockerfile)
    [[ "$FORCE_REBUILD_IMAGE" == "true" ]] && cmd+=(--no-cache)
    cmd+=($DOCKER_DIR)

    if [[ "$VERBOSE" == "true" ]]; then
        printf "${BBLACK}%s" echo -e "${BBLACK}-------- Docker command --------${COLOR_OFF}"
        printf "${BBLACK}%s" echo -e "${GREEN}${cmd[@}${COLOR_OFF}"
    fi

    "${cmd[@]}"

    if [[ $? -eq 0 ]]; then
        printf "${BBLACK}${GREEN}%s${COLOR_OFF}" "Image $IMAGE_NAME:$IMAGE_TAG built successfully"
    else
        printf "${BBLACK}${RED}%s${COLOR_OFF}" "Failed to build image $IMAGE_NAME:$IMAGE_TAG"
        exit 1
    fi

    docker tag "$IMAGE_NAME:$IMAGE_TAG" "$IMAGE_NAME:latest"
}

reset_container() {
    echo -e "${BBLACK}${YELLOW}Resetting container...${COLOR_OFF}"
    ARGS=""

    echo "FORCE_REBUILD_IMAGE: $FORCE_REBUILD_IMAGE"
    echo "FORCE_RESET_CONTAINER: $FORCE_RESET_CONTAINER"
    if [[ "$FORCE_REBUILD_IMAGE" == "true" ]]; then
        ARGS="--build-no-cache"
    fi

    if [[ "$FORCE_RESET_CONTAINER" == "true" ]]; then
        ARGS="$ARGS --remove-existing-container"
    fi

    echo "$DEVCONTAINER_BIN up $ARGS"
    $DEVCONTAINER_BIN up $ARGS
}

start_container() {
    local docker_instance=$(docker_instance)
    echo "$docker_instance"
    if [[ -z "$docker_instance" ]]; then
        local cmd=(docker run --rm -it)
        [[ "$RUN_PRIVILEGED" == "true" ]] && cmd+=(--privileged)

        # Add volume mounts to the command
        for mount in "${MOUNTS[@]}"; do
            cmd+=(-v "$mount")
        done
        cmd+=($ADDITIONAL_ARGS)
        [[ -n "$CONTAINER_NAME" ]] && cmd+=(--name "$CONTAINER_NAME")

        cmd+=(--tmpfs /tmp --tmpfs /run)
        # --cpus="2.0" --memory="32g" --memory-swap=-1 --memory-reservation="16g"

        cmd+=(-d "$IMAGE_NAME" /sbin/init)

        if [[ "$VERBOSE" == "true" ]]; then
            echo_color "BBLACK" "-------- Docker command --------"
            echo_color "GREEN" "${cmd[@]}"
        fi

        # Execute the command
        "${cmd[@]}"

        sleep 2
    fi
}

exec_instance() {
    local docker_instance=$(docker_instance)
    if [[ -z "$docker_instance" ]]; then
        printf "${BRED}No container found${COLOR_OFF}"
        exit 1
    fi
    docker exec -it ${docker_instance} /usr/bin/zsh
}


parse_opts() {
    local opt
    while getopts "n:vfr" opt; do
        case ${opt} in
            v ) VERBOSE="true" ;;
            f ) FORCE_REBUILD_IMAGE="false" ;;
            r ) FORCE_RESET_CONTAINER="false" ;;
            \? ) echo "Invalid option: $OPTARG" 1>&2; exit 1 ;;
        esac
    done
}

help() {
    echo -e "${BGREEN}Usage: $(basename "$0") [options] <command>${COLOR_OFF}
Options:
  -v  Verbose mode
  -f  Disable rebuilding the Docker image
  -r  Disable resetting the container

Commands:
  ${BGREEN}build${COLOR_OFF}             Build the Docker image
  ${BGREEN}start${COLOR_OFF}             Start the Docker container
  ${BGREEN}exec${COLOR_OFF}              Exec into the container
  ${BGREEN}reset${COLOR_OFF}             Reset the container
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
        build) build_image ;;
        start) start_container ;;
        exec) exec_instance ;;
        reset) reset_container ;;
        instance) docker_instance ;;
        *) help ;;
    esac
}

main "$@"
