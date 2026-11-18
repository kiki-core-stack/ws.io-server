#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -P -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

git fetch https://github.com/kiki-core-stack/ws.io-server main
git merge FETCH_HEAD
