#!/bin/bash

# Change to xxfunc directory, install the tool, and return to the original directory
(cd cargo-xxfunc && cargo install -f --path .) || { echo "Failed to install xxfunc"; exit 1; }

# Change to example directory, build the project, and return to the original directory
(cd example && cargo-xxfunc build --release) || { echo "Failed to build example project"; exit 1; }

# Change to runtime directory, run tests, and return to the original directory
(cd runtime && cargo test) || { echo "Tests failed in runtime directory"; exit 1; }
