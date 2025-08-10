#!/usr/bin/env bash

cargo run config-agent1.toml &
cargo run config-agent2.toml &
cargo run config-agent3.toml &

fg
