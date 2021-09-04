#!/usr/bin/env bash

docker run --volume $(realpath db/seed.sql):/docker-entrypoint-initdb.d/seed.sql --rm --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres
