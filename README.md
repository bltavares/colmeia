# colmeia

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well. Vaporware and might never be finished. Contributions welcome.

## Goals

Write a pure rust network stack compatible with dat, to be able to run it on desktop (Win/Mac/Linux), mobile (and maybe wasm).

## Milestones

- [x] Generate a binary that finds and talk to a LAN `dat` node: handshake and disconnect
- [x] Compile to Android
- [ ] **next** Create a connection pool that tracks dat peers, and track hypercore version to allow crossing bridges
- [ ] Bundle the static library into a Flutter app (**validated as viable already**) that displays the connection pool of dat peers for a given dat url (needs work)
- [ ] *(stretch goal)* **validated as viable already**  Write a Flutter app (micro-app as in a  micro-service) that syncs and share files, allowing to build other local-first apps without needing to bundle the network stack logic (like [dat-desktop](https://github.com/dat-land/dat-desktop) but for mobile and desktop without Node)
- [ ] *(stretch-er goal)* WASM and websocket integration
- [ ] *(stretch-er goal)* **validated as viable already** FFI bindings to be able to link into an iOS app and other languages ([neon](https://neon-bindings.com/)?/[helix](https://usehelix.com/)?/[dart ffi](https://users.rust-lang.org/t/ffi-support-in-dart/32375)?).


## References

### [dat 1](https://github.com/datproject/dat/tree/v13.13.1) compatibility

Modules:

- [x] `colmeia-dat1-mdns`: based on [dns-discovery](https://github.com/mafintosh/dns-discovery)
  - [x] `Locator`: stream to find dat members in the network
  - [x] `Announcer`: stream that announces a dat in the network
  - [x] `Mdns`: announces and find dat in the network
- [x] `colmeia-dat1-proto`: Parses DAT v1 wire protocol
  - [x] Handshake
  - [x] Read Encrypted
  - [x] Write Encrypted
  - [x] Add more methods and service handler
- [ ] **wip** `colmeia-dat1`
  - [x] Clone metadata into a hypercore feed
  - [x] Hypercore and Hyperfeed impl
  - [x] Use metadata to clone content hypercore feed ([first block is content public key](https://github.com/mafintosh/hyperdrive/blob/v9/index.js#L893-L896))
  - [x] Create a `Dat` struct that discovers and creates peeredfeed interactions
  - [ ] Write to disk
  - [ ] Impl remaining replicate logic
- [ ] `colmeia-dat1-dns` ?: DNS discovery based on [dns-discovery](https://github.com/mafintosh/dns-discovery)
- [ ] `colmeia-dat1-utp` ?: BitTorrent discover based on [discovery-channel](https://github.com/maxogden/discovery-channel)

Versions for reference:

- [dat 13.13.1](https://github.com/datproject/dat/tree/v13.13.1)
- [dat-node 3.5.15](https://github.com/datproject/dat-node/tree/v3.5.15)
- [discovery-swarm 5.1.4](https://github.com/mafintosh/discovery-swarm/tree/v5.1.4)
- [discovery-channel 5.5.1](https://github.com/maxogden/discovery-channel/tree/v5.5.1)
- [dns-discovery 6.2.3](https://github.com/mafintosh/dns-discovery/tree/v6.2.3)
- [hypercore-protocol 6.9.0](https://github.com/mafintosh/hypercore-protocol/tree/v6.9.0)
- [hypercore 7.7.1](https://github.com/mafintosh/hypercore/tree/v7.7.1)
- [hyperdrive 9.14.5](https://github.com/mafintosh/hyperdrive/tree/v9.14.5)
- [How dat works](https://datprotocol.github.io/how-dat-works/) website

### [hyperswarm](https://github.com/hyperswarm) & dat 2 compatibility

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

## Relevant patches upstream

- [ ] [Make merkle-treem-stream Send](https://github.com/datrs/merkle-tree-stream/pull/28)
- [ ] [Make hypercore Send](https://github.com/datrs/hypercore/pull/95)
- [ ] [Remove internal buffering on simple-message-channel to avoid bytes being dropped during handshake](https://github.com/datrs/simple-message-channels/pull/5)
- Bumps to make `datrs/hypercore` compatible with `rand` and `-dalek` pre release (No patches yet)

## Utils

```
Local dat2: be41e3d43d054982e14dfc60281d9d4425ab5d4b0b280a355b7927869ca08fc5
```

### Find a peer, download stuff

```sh
RUST_LOG="colmeia_dat1=debug" cargo run --bin colmeia-sync -- dat://642b2da5e4267635259152eb0b1c04416030a891acd65d6c942b8227b8cbabed
 ```


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

### Clone from a single host

```sh
RUST_LOG=debug cargo run --bin colmeia-clone -- 192.168.15.173:3282 dat://6268b99fbacacea49c6bc3d4776b606db2aeadb3fa831342ba9f70d55c98929f
```

### dat server debug mode

```sh
DEBUG="dat*" npx dat share
```

## Android

### Direct compilation

- **No root required**

Inside termux, install `git` and `rust`:

```sh
apk install git
apk install rust
git clone https://github.com/bltavares/colmeia
cd colmeia

cargo run # your desired bin
```

### Cross Compilation

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

### Flutter

#### Android libs

Only runs on Linux (or WSL2)

```sh
make android
```

#### iOS libs

Only works on Mac.
`Enable Bitcode = false`

```sh
# 64 bit targets (real device & simulator):
rustup target add aarch64-apple-ios x86_64-apple-ios
cargo install cargo-lipo
```

```sh
make ios
```
