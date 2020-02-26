#!/bin/bash -x

sleep 3
diesel migration run

exec "$@"
