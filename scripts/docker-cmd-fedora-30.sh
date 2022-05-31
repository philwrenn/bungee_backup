#!/bin/bash
DIR="$( realpath "$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )/.." )"
USER_ID="$(id -u)"

DOCKER_CMD_BASE="docker run --rm -ti \
    -v $DIR:/bungee_backup \
    -e "USER_ID=$USER_ID" \
    -w "/bungee_backup" \
    bungee_backup_build:fedora-30"
$DOCKER_CMD_BASE $@