# surrealdb.node

The official SurrealDB library for Node.js.

[![](https://img.shields.io/badge/status-beta-ff00bb.svg?style=flat-square)](https://github.com/surrealdb/surrealdb.node) [![](https://img.shields.io/badge/docs-view-44cc11.svg?style=flat-square)](https://surrealdb.com/docs/integration/libraries/nodejs) [![](https://img.shields.io/badge/license-Apache_License_2.0-00bfff.svg?style=flat-square)](https://github.com/surrealdb/surrealdb.node)


<h2><img height="20" src="https://github.com/surrealdb/surrealdb/raw/main/img/whatissurreal.svg?raw=true">&nbsp;&nbsp;What is SurrealDB?</h2>

SurrealDB is an end-to-end cloud-native database designed for modern applications, including web, mobile, serverless, Jamstack, backend, and traditional applications. With SurrealDB, you can simplify your database and API infrastructure, reduce development time, and build secure, performant apps quickly and cost-effectively.

**Key features of SurrealDB include:**

- **Reduces development time**: SurrealDB simplifies your database and API stack by removing the need for most server-side components, allowing you to build secure, performant apps faster and cheaper.
- **Real-time collaborative API backend service:** SurrealDB functions as both a database and an API backend service, enabling real-time collaboration.
- **Support for multiple querying languages:** SurrealDB supports SQL querying from client devices, GraphQL, ACID transactions, WebSocket connections, structured and unstructured data, graph querying, full-text indexing, and geospatial querying.
- **Granular access control**: SurrealDB provides row-level permissions-based access control, giving you the ability to manage data access with precision.


View the [features](https://surrealdb.com/features), the latest [releases](https://surrealdb.com/releases), the product [roadmap](https://surrealdb.com/roadmap), and [documentation](https://surrealdb.com/docs).


<h2><img height="20" src="https://github.com/surrealdb/surrealdb/blob/main/img/gettingstarted.svg?raw=true">&nbsp;&nbsp;Getting started</h2>

```js
// import as ES module or common JS
import {Surreal} from 'surrealdb.node';
const {Surreal} = require('surrealdb.node');

const db = new Surreal();

// use in-memory database
await db.connect('memory');
// connect to database server
await db.connect('ws://127.0.0.1:8000');
// use rocksdb file
await db.connect(`rocksdb://${process.cwd()}/test.db`);

// Select a specific namespace / database
await db.use({ ns: "test", db: "test" });

// Create a new person with a random id
let created = await db.create("person", {
    title: "Founder & CEO",
    name: {
        first: "Tobie",
        last: "Morgan Hitchcock",
    },
    marketing: true,
    identifier: Math.random().toString(36).substr(2, 10),
});

// Update a person record with a specific id
let updated = await db.merge("person:jaime", {
    marketing: true
});

// Select all people records
let people = await db.select("person");

// Perform a custom advanced query
let groups = await db.query(
    "SELECT marketing, count() FROM type::table($table) GROUP BY marketing",
    {
        table: "person",
    },
);

// Delete all people upto but not including Jaime
let deleted = await db.delete("person:..jaime");

// Delete all people
await db.delete("person");
```

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
