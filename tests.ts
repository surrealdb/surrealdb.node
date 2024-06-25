import Surreal from "surrealdb.js";
import { surrealdbNodeEngines } from "./lib-src/embedded.ts";

const surreal = new Surreal({
    engines: surrealdbNodeEngines()
});

// console.log("connecting", await surreal.connect("mem://"));

// console.log("using", await surreal.use({ namespace: "test", database: "test" }));

// console.log("listening", await surreal.live("test", (res) => console.log("recieved live" + res)));

// console.log("creating", await surreal.create('test', { val: 42 }));

// console.log("closing", await surreal.close());