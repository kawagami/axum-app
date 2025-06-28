#!/bin/bash

code . && docker-compose up -d && cargo watch -x run
