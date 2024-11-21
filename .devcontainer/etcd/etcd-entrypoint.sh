#!/bin/bash

# Ensure the data directory exists
mkdir -p /etcd/data

# Run the setup script in the background
/usr/local/bin/setup_etcd.sh &

# Execute the CMD
exec "$@"