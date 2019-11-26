#! /bin/bash

docker exec $1 /usr/local/bin/pg_dump postgres://bba:MySeCrEtPaSsWoRd@localhost/bba --compress 9 > ./seed.sql.gz
