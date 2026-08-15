#!/usr/bin/env bash

printf "\nInstalling Go dependencies...\n";
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
export PATH="$PATH:$(go env GOPATH)/bin"

printf "\nInstalling Python dependencies...\n";
python3 -m pip install --upgrade pip
python3 -m pip install grpcio grpcio-tools
