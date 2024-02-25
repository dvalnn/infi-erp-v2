#!/bin/bash
docker run --rm --name dev-db -e POSTGRES_PASSWORD=postgrespw -p 5432:5432 -d postgres
