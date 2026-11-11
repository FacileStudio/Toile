#!/usr/bin/env sh
set -eu
container="toile_dev_postgres"
if [ "$(docker inspect -f '{{.State.Running}}' "$container" 2>/dev/null || echo missing)" = "true" ]; then
  docker stop "$container" >/dev/null
  echo "stopped toile_dev_postgres"
else
  echo "toile_dev_postgres is not running"
fi
