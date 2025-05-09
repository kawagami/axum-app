#!/bin/bash

ACCOUNT=kawagami77
IMAGENAME=axum-app
TAG=latest

# 先更新 sqlx prepare 資料然後再 build
cargo sqlx prepare && docker build --no-cache -t $ACCOUNT/$IMAGENAME:$TAG .
