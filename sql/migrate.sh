#!/usr/bin/env sh

docker run --rm -v \
  "$(PWD)/sql:/flyway/sql" \
  -v "$(PWD)/conf:/flyway/conf" \
  -v "$(PWD)/../db:/flyway/db" \
  flyway/flyway migrate

