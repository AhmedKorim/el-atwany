#!/usr/bin/env bash

# Set an error handler
trap on_exit EXIT

# printing the simple stack trace
on_exit() {
    while caller $((n++));
    do :;
    done;
}

CONTAINER_NAME=atwany
IMAGE_VERSION=0.1.0
IMAGE_NAME=atwany
DOCKET_FILE=ATWANY.Dockerfile
echo 'Starting..'
echo 'Removing old Container if any ! ..'
docker stop ${CONTAINER_NAME}
docker rm ${CONTAINER_NAME}
echo "Stopped ${CONTAINER_NAME}"
echo 'Building Docker Image, wait ..'
docker build -t el-atwany/${IMAGE_NAME}:${IMAGE_VERSION} -f ${DOCKET_FILE} .
