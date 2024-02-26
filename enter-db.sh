#!/bin/bash
docker exec -it dev-db psql -h localhost -p 5432 -U postgres -W
