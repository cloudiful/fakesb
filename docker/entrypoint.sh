#!/usr/bin/env bash
set -euo pipefail

/usr/local/bin/fakESB &
server_pid=$!

nginx -g 'daemon off;' &
nginx_pid=$!

cleanup() {
  kill "$server_pid" "$nginx_pid" 2>/dev/null || true
}

trap cleanup EXIT INT TERM
wait -n "$server_pid" "$nginx_pid"
cleanup
