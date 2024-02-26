#!/bin/bash
docker exec -it infi-erp-v2-postgres-1 psql -h localhost -p 5432 -U postgres -W
