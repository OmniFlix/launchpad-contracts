#!/bin/bash

# Define the main directory path
MAIN_DIR=$(dirname "$0")/..
CONTRACTS_DIR="$MAIN_DIR/contracts"


# Iterate over directories within contracts/factories
for d in "$CONTRACTS_DIR"/factories/*; do
  if [ -d "$d" ]; then
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# Iterate over directories within contracts/minters
for d in "$CONTRACTS_DIR"/minters/*; do
  if [ -d "$d" ]; then
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# Iterate over directories within contracts/whitelists
for d in "$CONTRACTS_DIR"/whitelists/*; do
  if [ -d "$d" ]; then
    cd "$d" || exit
    cargo schema
    cd "$MAIN_DIR" || exit
  fi
done

# go main directory/ts
cd "$MAIN_DIR"/ts || exit
yarn generate-ts
cd "$MAIN_DIR" || exit
```

# Iterate over directories within contracts/factories
for d in "$CONTRACTS_DIR"/factories/*; do
  if [ -d "$d" ]; then
  cd "$d" || exit
  rm -rf "$d"/schema
    cd "$MAIN_DIR" || exit
  done
  fi
done

# Iterate over directories within contracts/minters
for d in "$CONTRACTS_DIR"/minters/*; do
  if [ -d "$d" ]; then
    cd "$d" || exit
    rm -rf "$d"/schema
    cd "$MAIN_DIR" || exit
  fi
done

# Iterate over directories within contracts/whitelists
for d in "$CONTRACTS_DIR"/whitelists/*; do
  if [ -d "$d" ]; then
    cd "$d" || exit
    rm -rf "$d"/schema
    cd "$MAIN_DIR" || exit
  fi
done

