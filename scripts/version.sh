#!/bin/bash
# shellcheck disable=SC2034
VERSION="$(grep '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')"
