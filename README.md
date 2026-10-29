<p align="center">
  <img src="https://avatars.githubusercontent.com/u/349028731?s=400&u=06d34ce9df4dea7dc727926d1272481eb1f92614&v=4" alt="RePorted-Engine" width="128" height="128">
</p>

<h1 align="center">Crabbed</h1>

<p align="center">
  Minecraft: Java Edition 26.3 rewritten in Rust
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Minecraft-Java%2026.3-5D9C3F" alt="Minecraft Java 26.3">
  <img src="https://img.shields.io/badge/Rust-stable-B7410E?logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/license-MIT-3B2F7E" alt="MIT license">
</p>

Crabbed is a fork of [Horizon](https://github.com/ludovic111/horizon), originally created and developed by [Ludovic Marie](https://github.com/ludovic111). 

This fork aims to keep the same goal: a native Minecraft: Java Edition 26.3 client and server rewritten in Rust with a focus on vanilla parity.

> Not an official Minecraft product. Not approved by or associated with Mojang or Microsoft. This project contains no Minecraft code, assets or data - you need your own legally obtained copy of Minecraft: Java Edition 26.3.

## Credits

- Original project: [Horizon](https://github.com/ludovic111/horizon) by [Ludovic Marie](https://lu4ovic.com)

## Build from source

Requirements: Rust 1.90+, Python 3.10+, JDK 25, and on Linux the ALSA and udev development headers.

This project builds its resources from your own copy of Minecraft: Java Edition 26.3.

1. Install Java Edition 26.3 with the official launcher and play it once
2. Obtain the 26.3 server jar from the launcher or [minecraft.net/download/server](https://www.minecraft.net/en-us/download/server)

```sh
git clone https://github.com/RePorted-Engine/Crabbed.git
cd Crabbed
cargo run --release -p horizon-prepare -- --prepare --server-jar ~/Downloads/server.jar
cargo run --release -p horizon-client
```

## License

[MIT](LICENSE), same as the original Horizon project. Minecraft is a trademark of Mojang Synergies AB. Minecraft code and assets belong to Mojang and Microsoft and are covered by the [Minecraft EULA](https://www.minecraft.net/en-us/eula).
