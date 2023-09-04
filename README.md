# surrealdb.node

The official SurrealDB library for Node.js.

[![](https://img.shields.io/badge/status-beta-ff00bb.svg?style=flat-square)](https://github.com/surrealdb/surrealdb.node) [![](https://img.shields.io/badge/docs-view-44cc11.svg?style=flat-square)](https://surrealdb.com/docs/integration/libraries/nodejs) [![](https://img.shields.io/badge/license-Apache_License_2.0-00bfff.svg?style=flat-square)](https://github.com/surrealdb/surrealdb.node)


# Supported targets

| Tripple                       | supported | rocksdb support |           reason |
| ----------------------------- | :-------: | :-------------: | ---------------: |
| aarch64-apple-darwin          |     ✓     |        x        |                  |
| aarch64-linux-android         |     ✓     |        ✓        |                  |
| aarch64-unknown-linux-gnu     |     ✓     |        ✓        |                  |
| aarch64-unknown-linux-musl    |     ✓     |        x        |                  |
| aarch64-pc-windows-msvc       |     x     |        x        | ring build fails |
| armv7-unknown-linux-gnueabihf |     x     |        x        |  psm build fails |
| x86_64-unknown-linux-musl     |     ✓     |        x        |                  |
| x86_64-unknown-freebsd        |     ✓     |        x        |                  |
| i686-pc-windows-msvc          |     ✓     |        ✓        |                  |
| armv7-linux-androideabi       |     ✓     |        ✓        |                  |
| universal-apple-darwi         |     ✓     |        x        |                  |
