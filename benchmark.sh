#!/bin/bash
ITERATIONS=100

echo "Benchmarking multiple mkdir..."
time for i in $(seq 1 $ITERATIONS); do
    DIR_BASE="/tmp/test_mkdir_multi_$i"
    mkdir -p "${DIR_BASE}/mailserver/mail-data"
    mkdir -p "${DIR_BASE}/mailserver/mail-state"
    mkdir -p "${DIR_BASE}/mailserver/mail-logs"
    mkdir -p "${DIR_BASE}/mailserver/config"
    mkdir -p "${DIR_BASE}/roundcube/db"
    mkdir -p "${DIR_BASE}/roundcube/config"
    mkdir -p "${DIR_BASE}/media"
done

echo "Benchmarking single mkdir..."
time for i in $(seq 1 $ITERATIONS); do
    DIR_BASE="/tmp/test_mkdir_single_$i"
    mkdir -p \
        "${DIR_BASE}/mailserver/mail-data" \
        "${DIR_BASE}/mailserver/mail-state" \
        "${DIR_BASE}/mailserver/mail-logs" \
        "${DIR_BASE}/mailserver/config" \
        "${DIR_BASE}/roundcube/db" \
        "${DIR_BASE}/roundcube/config" \
        "${DIR_BASE}/media"
done
