# colmeia

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well. Vaporware and might never be finished. Contributions welcome.

## Goals

Write a pure rust network stack compatible with dat, to be able to run it on desktop (Win/Mac/Linux), mobile (and maybe wasm).

## Milestones

- [x] Generate a binary that finds and talk to a LAN `dat` node: handshake and disconnect
- [x] Compile to Android
- [ ] **next** Create a connection pool that tracks dat peers, and track hypercore version to allow crossing bridges
- [ ] Bundle the binary into a Flutter app that displays the connection pool of dat peers for a given dat url
- [ ] *(stretch goal)* Write a Flutter app (micro-app as in a  micro-service) that syncs and share files, allowing to build other local-first apps without needing to bundle the network stack logic (like [dat-desktop](https://github.com/dat-land/dat-desktop) but for mobile and desktop without Node)
- [ ] *(stretch-er goal)* WASM and websocket integration
- [ ] *(stretch-er goal)* FFI bindings to be able to link into an iOS app and other languages ([neon](https://neon-bindings.com/)?/[helix](https://usehelix.com/)?/[dart ffi](https://users.rust-lang.org/t/ffi-support-in-dart/32375)?).

## References

### [dat](https://github.com/datproject/dat/tree/v13.13.1) compatibility

Modules:

- [x] `colmeia-dat-mdns`: based on [dns-discovery](https://github.com/mafintosh/dns-discovery)
  - [x] `Locator`: stream to find dat members in the network
  - [x] `Announcer`: stream that announces a dat in the network
  - [x] `Mdns`: announces and find dat in the network
- [x] `colmeia-dat-proto`: Parses DAT v1 wire protocol
  - [x] Handshake
  - [x] Read Encrypted
  - [x] Write Encrypted
  - [x] Add more methods and service handler
  - [ ] **next** Integrate with hypercore
- [ ] `colmeia-dat-dns` ?: DNS discovery based on [dns-discovery](https://github.com/mafintosh/dns-discovery)
- [ ] `colmeia-dat-utp` ?: BitTorrent discover based on [discovery-channel](https://github.com/maxogden/discovery-channel)

Versions for reference:

- [dat 13.13.1](https://github.com/datproject/dat/tree/v13.13.1)
- [dat-node 3.5.15](https://github.com/datproject/dat-node/tree/v3.5.15)
- [discovery-swarm 5.1.4](https://github.com/mafintosh/discovery-swarm/tree/v5.1.4)
- [discovery-channel 5.5.1](https://github.com/maxogden/discovery-channel/tree/v5.5.1)
- [dns-discovery 6.2.3](https://github.com/mafintosh/dns-discovery/tree/v6.2.3)
- [hypercore-protocol 6.9.0](https://github.com/mafintosh/hypercore-protocol/tree/v6.9.0)
- [How dat works](https://datprotocol.github.io/how-dat-works/) website

### [hyperswarm](https://github.com/hyperswarm) compatibility

New protocol is in development. No stable release of either beaker or `dat` available yet.
Uses NOISE protocol and a different handshake.

Reference tools:

- [hyperdrive-daemon](https://github.com/andrewosh/hyperdrive-daemon)
- [hyperdrive v10](https://github.com/mafintosh/hyperdrive)
- [snow noise protocol](https://snow.rs/)
- [hypercore-protocol-rust-experiments](https://github.com/Frando/hypercore-protocol-rust-experiments)

Modules:

- [ ] `colmeia-dht`: Interop with hypwerswarm dht infrastructure
- [ ] `colmeia-mdns`: Support to hypwerswarm mdns infrastructure
- [ ] `colmeia-hypercore-proto`: Dat v2 wire protocol

Versions for reference: **TDB**

### General

- [ ] `colmeia-resolve`: converts a `.well-known/dat`  or DNS TXT [into a hash](https://beakerbrowser.com/docs/guides/use-a-domain-name-with-dat)
- [ ] `colmeia-network`: Network connection pool manager, allowing to use other modules to fetch information

## Utils

### Find a local peer

[colmeia-mdns](./src/bin/colmeia-mdns.rs)

```sh
RUST_LOG=debug cargo run --bin colmeia-mdns -- dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```

### Connect and troubleshoot a peer

[colmeia-nc](./src/bin/colmeia-nc.rs)

```sh
RUST_LOG=debug cargo run --bin colmeia-nc -- 127.0.0.1:3282  dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```

### Start a server (WIP)

```sh
RUST_LOG=debug cargo run --bin colmeia-server -- 0.0.0.0:8787 dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```

### dat server debug mode

```sh
DEBUG="dat*" npx dat share
```

## Cross compilation

### Android

[It works!](https://twitter.com/bltavares/status/1221587189668163584)

- **No root required**
- Windows: Using WSL2 (windows insider) allows you to use Docker from WSL. [Follow this guide to enable docker](https://docs.docker.com/docker-for-windows/wsl-tech-preview/)
- Mac/Linux: It should just need to have docker configured properly

- Install [cross](https://github.com/rust-embedded/cross#installation)

- Cross compile to your target:

```sh
cross build --target armv7-linux-androideabi
```

- To find the correct target for your device: `adb shell uname -m`

| arch             | target                  |
|------------------|-------------------------|
| armv7l           | armv7-linux-androideabi |
| aarch64 or arm64 | aarch64-linux-android   |
| arm*             | arm-linux-androideabi   |

*arm: ([seems to be broken](https://internals.rust-lang.org/t/what-is-the-current-status-of-arm-linux-androideabi/4507/7))

- Push to Android on a writeable location. Android does not allow binaries to be executed from the `/sdcard`:

```sh
# Once
adb shell mkdir -p /data/local/tmp/colmeia
adb shell chmod 777 /data/local/tmp/colmeia # Make it readable to let Termux read it as well

adb push ./target/armv7-linux-androideabi/debug/colmeia-mdns /data/local/tmp/colmeia
```

### Android ADB shell

```sh
adb shell # open shell in android
chmod +x /data/local/tmp/colmeia/colmeia-server

RUST_LOG=debug  /data/local/tmp/colmeia/colmeia-server 0.0.0.0:8787 dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```

### [Termux](https://termux.com/) from inside Android

Android don't allow execution from outside the app data dir, and you can't push data to other apps dir from `adb push`. You need to copy the binary to your root folder, which should be executable.

Open the app and copy the content to the home folder:

```sh
# Inside termux shell
cp /data/local/tmp/colmeia/colmeia-server ~

RUST_LOG=debug ~/colmeia-server 0.0.0.0:8787 dat://460f04cf12c3b9833e5a0d3dd8eea05eab59dd8c1438a7454afe9630b9b4f8bd
```
