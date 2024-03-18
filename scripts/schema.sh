#!/bin/bash

# Define the main directory path
MAIN_DIR=$(dirname "$(realpath "$0")")/..

# Print out the main directory path for debugging
echo "Main directory: $MAIN_DIR"

CONTRACTS_DIR="$MAIN_DIR/contracts"
TS_DIR="$MAIN_DIR/ts"

# Print out the contracts directory path for debugging
echo "Contracts directory: $CONTRACTS_DIR"

# Iterate over directories within contracts/factories
for d in "$CONTRACTS_DIR"/factories/*; do
  if [ -d "$d" ]; then
    echo "Processing directory: $d"
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# Iterate over directories within contracts/minters
for d in "$CONTRACTS_DIR"/minters/*; do
  if [ -d "$d" ]; then
    echo "Processing directory: $d"
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# Iterate over directories within contracts/whitelists
for d in "$CONTRACTS_DIR"/whitelists/*; do
  if [ -d "$d" ]; then
    echo "Processing directory: $d"
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# Print out the ts directory path for debugging
echo "TS directory: $TS_DIR"

# go main directory/ts
cd "$TS_DIR" || exit
yarn generate-ts
cd "$MAIN_DIR" || exit

# Iterate over directories within contracts/factories to clean schema files
for d in "$CONTRACTS_DIR"/factories/*; do
  if [ -d "$d" ]; then
    echo "Cleaning schema files in directory: $d"
    rm -rf "$d"/schema
  fi
done

# Iterate over directories within contracts/minters to clean schema files
for d in "$CONTRACTS_DIR"/minters/*; do
  if [ -d "$d" ]; then
    echo "Cleaning schema files in directory: $d"
    rm -rf "$d"/schema
  fi
done

# Iterate over directories within contracts/whitelists to clean schema files
for d in "$CONTRACTS_DIR"/whitelists/*; do
  if [ -d "$d" ]; then
    echo "Cleaning schema files in directory: $d"
    rm -rf "$d"/schema
  fi
done
