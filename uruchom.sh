#!/bin/bash
cd "$(dirname "$(readlink -f "$0")")"
./src-tauri/target/release/document-processor
