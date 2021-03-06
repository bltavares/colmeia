# colmeia

<a href="https://github.com/bltavares/colmeia/actions?query=workflow%3AQuickstart+branch%3Amaster">
    <img src="https://img.shields.io/github/workflow/status/bltavares/colmeia/Quickstart/master?label=main%20ci" />
</a>
<a href="https://github.com/bltavares/colmeia/actions?query=workflow%3ACross-compile+branch%3Amaster">
    <img src="https://img.shields.io/github/workflow/status/bltavares/colmeia/Cross-compile/master?label=cross%20ci" />
</a>

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well. Vaporware and might never be finished. Contributions welcome.

## Goals

Write a pure rust network stack compatible with dat, to be able to run it on desktop (Win/Mac/Linux), mobile (and maybe wasm).

## Milestones

- [x] Generate a binary that finds and talk to a LAN `dat` node: handshake and disconnect
- [x] Compile to Android
- [ ] **next** Create a connection pool that tracks dat peers, and track hypercore version to allow crossing bridges
- [ ] Bundle the static library into a Flutter app (**validated as viable already with dat (legacy)**) that displays the connection pool of dat peers for a given dat url (needs work)
- [x] **(strech goal)** Support routers (MIPS)
- [ ] *(stretch goal)* **validated as viable already with dat (legacy)**  Write a Flutter app (micro-app as in a  micro-service) that syncs and share files, allowing to build other local-first apps without needing to bundle the network stack logic (like [dat-desktop](https://github.com/dat-land/dat-desktop) but for mobile and desktop without Node)
- [ ] *(stretch-er goal)* WASM and websocket integration
- [ ] *(stretch-er goal)* **validated as viable already with dat (legacy)** FFI bindings to be able to link into an iOS app and other languages ([neon](https://neon-bindings.com/)?/[helix](https://usehelix.com/)?/[dart ffi](https://users.rust-lang.org/t/ffi-support-in-dart/32375)?).

## References

### [hyperswarm](https://github.com/hyperswarm) & [hypercore-protocol](https://hypercore-protocol.org/) compatibility

Uses NOISE protocol and a different handshake.

#### Modules

#### Discovery

- [x] `colmeia-hyperswarm-mdns`: Support to hyperswarm mdns infrastructure: based on [hyperswarm/discovery](https://github.com/hyperswarm/discovery/)
  - [x] `Locator`: stream to find dat members in the network
  - [x] `Announcer`: stream that announces a dat in the network
  - [x] `Mdns`: announces and find dat in the network
  - [ ] Tests
  - [ ] All protocol 1:1
- [ ] `colmeia-dht`: Interop with hypwerswarm dht infrastructure (:eyes: <https://github.com/mattsse/hyperswarm-dht>)
  - [ ] Tests
  - [ ] All protocol 1:1

#### Protocol

- [x] **wip** `colmeia-hypercore`: Networked module for hypercore storage
  - [x] Hypercore replicate logic
  - [ ] Tests
  - [ ] All protocol 1:1
- [x] **wip** `colmeia-hyperdrive`: Networked implementation of content and metadata hypercore's
  - [x] `Hyperdrive`: Use metadata to clone content hypercore feed ([first block is content public key](https://github.com/hypercore-protocol/hyperdrive/blob/v10.13.0/index.js#L186))
  - [x] In memory
  - [ ] Write to disk
  - [x] Impl remaining replicate logic for Hyperdrives
  - [ ] Tests
  - [ ] All protocol 1:1
- [x] **wip** `colmeia-hyperstack`: Discovery integration of hyperdrives
  - [x] Create a `Hypercore` struct that discovers and creates peeredfeed interactions
  - [ ] Tests


#### Reference tools

- [hyperdrive-daemon](https://github.com/andrewosh/hyperdrive-daemon)
- [hyperdrive v10](https://github.com/mafintosh/hyperdrive)
- [snow noise protocol](https://snow.rs/)
- [hypercore-protocol-rust-experiments](https://github.com/Frando/hypercore-protocol-rust-experiments)
- [hypercore-protocol-rs](https://github.com/Frando/hypercore-protocol-rs)
- [Beaker Browser](https://beakerbrowser.com/)

#### Versions for reference

- [hypercore v10.13.0](https://github.com/hypercore-protocol/hyperdrive/blob/v10.13.0/index.js)
- [hypercore-protocol spec](https://hypercore-protocol.org/)
- **TDB**

## CLI binaries

```txt
Local dat2: be41e3d43d054982e14dfc60281d9d4425ab5d4b0b280a355b7927869ca08fc5
```

### Find a local peer

[colmeia-mdns](./colmeia-bins/src/bin/colmeia-hyperswarm-mdns.rs)

```sh
RUST_LOG=debug cargo run --bin colmeia-hyperswarm-mdns -- 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5 8000
```

### Connect and troubleshoot a peer

[colmeia-nc](./colmeia-bins/src/bin/colmeia-nc.rs)

```sh
RUST_LOG=debug cargo run --bin colmeia-nc -- 7e5998407b3d9dbb94db21ff50ad6f1b1d2c79e476fbaf9856c342eb4382e7f5 127.0.0.1:3282
```

### Clone from a single host

```sh
RUST_LOG=debug cargo run --bin colmeia-clone -- 192.168.15.173:3282 dat://6268b99fbacacea49c6bc3d4776b606db2aeadb3fa831342ba9f70d55c98929f
```

## Platforms

:warning: **TODO**: redo support as part of dat -> hypercore migration

### Android Direct compilation

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
| mips-musl        | mips-unknown-linux-musl |

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

### OpenWRT

It compiles with OpenWRT. Tested with 2 different routers on `18.06.1` and `19.07.2`.

Run and `scp` the binaries on `target/mips-unknown-linux-musl`:

```sh
 cross build --target mips-unknown-linux-musl
 ```
