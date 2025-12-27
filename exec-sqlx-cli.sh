#!/bin/bash

UID=$(id -u) GID=$(id -g) docker-compose run --rm sqlx-cli $@
