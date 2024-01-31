#!/bin/bash

# Loop 10 times
for ((i = 0; i < 100; i++)); do
    key="key$i"
    value="value$i"
    json="{\"key\": \"$key\", \"value\": \"$value\"}"

    # Run your command in the background with &
    grpcurl -plaintext -proto proto/keyvalue.proto -d "$json" localhost:50051 keyvalue.KeyValue/Store &
done

wait

for ((i = 0; i < 100; i++)); do
    key="key$i"
    json="{\"key\": \"$key\"}"

    # Run your command in the background with &
    grpcurl -plaintext -proto proto/keyvalue.proto -d "$json" localhost:50051 keyvalue.KeyValue/Retrieve &
done

# Wait for all background processes to finish
wait

