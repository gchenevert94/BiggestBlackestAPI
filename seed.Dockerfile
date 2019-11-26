FROM postgres:9.6.6-alpine
COPY raw_dump.sql.gz /docker-entrypoint-initdb.d/
RUN chmod a+r /docker-entrypoint-initdb.d/*
