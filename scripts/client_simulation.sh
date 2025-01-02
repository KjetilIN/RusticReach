#!/bin/bash

# Run cargo build
echo "Starting cargo build..."
cargo build
if [ $? -ne 0 ]; then
  echo "Cargo build failed. Exiting."
  exit 1
fi

echo "Cargo build completed successfully."

# Start the client and redirect stdin
./target/debug/client -c client.yml &
CLIENT_PID=$!
echo "Client started with PID $CLIENT_PID"

# Allow the client to initialize
sleep 2

echo "Sending initialization commands to the client..."
# Interact with the client via stdin
{
  echo "/name User$(date +%s)"   # Set a unique name based on timestamp
  echo "/join commonchat"        # Join the commonchat
} > /proc/$CLIENT_PID/fd/0

# Define the file containing text messages
TEXT_FILE="text_message_simulation.txt"

# Check if the file exists
if [ ! -f "$TEXT_FILE" ]; then
  echo "Text file $TEXT_FILE not found. Exiting."
  kill $CLIENT_PID
  exit 1
fi

# Start sending random messages for 1 minute
echo "Starting to send random messages for 1 minute..."
END_TIME=$((SECONDS + 60))
while [ $SECONDS -lt $END_TIME ]; do
  # Pick a random line from the text file
  MESSAGE=$(shuf -n 1 "$TEXT_FILE")

  # Send the message to the client via stdin
  echo "$MESSAGE" > /proc/$CLIENT_PID/fd/0
  echo "Sent message: $MESSAGE"

  # Wait for a random time between 1 and 5 seconds
  SLEEP_TIME=$((RANDOM % 5 + 1))
  echo "Waiting for $SLEEP_TIME seconds before sending the next message..."
  sleep $SLEEP_TIME
done

# Clean up
echo "Stopping the client..."
kill $CLIENT_PID
echo "Simulation complete. Client stopped."
