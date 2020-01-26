# colmeia

----
Hive (in portuguese). Attempt to make an interop layer to connect to [dat](https://github.com/datrs/) on [hyperswarm](https://github.com/hyperswarm) and legacy infra as well. Vaporware and might never be finished. Contributions welcome.

## Goals

Write a pure rust network stack compatible with dat, to be able to run it on desktop (Win/Mac/Linux), mobile (and maybe wasm).

## Milestones

- [ ] **wip** Generate a binary that finds and talk to a LAN `dat` node: handshake and disconnect
- [ ] Create a connection pool that tracks dat peers
- [ ] Compile to Android
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
- [ ] `colmeia-dat-proto` **wip**: Parses DAT wire protocol
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

Modules:

- [ ] `colmeia-dht`: Interop with hypwerswarm dht infrastructure
- [ ] `colmeia-mdns`: Support to hypwerswarm mdns infrastructure

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
