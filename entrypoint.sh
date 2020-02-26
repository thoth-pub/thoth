#!/bin/bash -x

sleep 5
diesel migration run

exec "$@"
