#!/usr/bin/env bash

cargo run config-agent1.toml &
AGENT1=$!
cargo run config-agent2.toml &
AGENT2=$!
cargo run config-agent3.toml &
AGENT3=$!

kill_agents() {
    kill $AGENT1
    kill $AGENT2
    kill $AGENT3
    exit 0
}

trap 'kill_agents' SIGINT

print 'Agents running, CTRL+C to quit'
while true; do
    sleep 10
done
