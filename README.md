# stignore-agent
stignore agent v2 written in Rust

## Running
```
cargo run config.toml
```

## Containers
By default the container attempts to load `/app/config.toml`, if you don't want this just specify a different config file as the first parameter.

Build:
```
docker build -t stignore-agent:latest .
```

Run:
```
# Default config location
docker run -it --rm --name stignore-agent -v "$(pwd)/config-agent.toml:/app/config.toml" -p 9000:9000 -e RUST_LOG=info stignore-agent:latest

# Custom config location
docker run -it --rm --name stignore-agent -v "$(pwd)/config-agent.toml:/app/config-custom.toml" -p 9000:9000 -e RUST_LOG=info stignore-agent:latest /app/config-custom.toml
```
