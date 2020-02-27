#!/bin/bash -x

sleep 3
thoth migrate

exec "$@"
