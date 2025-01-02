#!/bin/bash

# Run cargo build
cargo build
if [ $? -ne 0 ]; then
  echo "Cargo build failed. Exiting."
  exit 1
fi

# Start the client and redirect stdin
./target/debug/client -c client.yml &
CLIENT_PID=$!

# Allow the client to initialize
sleep 400

# Interact with the client via stdin
{
  echo "/name User$(date +%s)"   # Set a unique name based on timestamp
  echo "/join commonchat"        # Join the commonchat
} > /proc/$CLIENT_PID/fd/0

exit 1
