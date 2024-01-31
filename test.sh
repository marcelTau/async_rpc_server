#/bin/bash

grpcurl -plaintext -proto proto/keyvalue.proto  -d '{"key": "123", "value": "some value"}' localhost:50051 keyvalue.KeyValue/Store

sleep 1

grpcurl -plaintext -proto proto/keyvalue.proto  -d '{"key": "123"}' localhost:50051 keyvalue.KeyValue/Retrieve


