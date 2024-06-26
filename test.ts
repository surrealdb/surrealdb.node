import Surreal from "surrealdb.js";
import { surrealdbNodeEngines } from "./lib-src/embedded.ts";

async function run_mem() {
    const surreal = new Surreal({
        engines: surrealdbNodeEngines()
    });

    console.log("connecting mem", await surreal.connect("mem://", { versionCheck: false }));

    console.log("using mem", await surreal.use({ namespace: "test", database: "test" }));

    console.log("listening mem", await surreal.live("test", (res) => console.log("recieved live" + res)));

    console.log("creating mem", await surreal.create('test', { val: 42 }));

    console.log("selecting mem", await surreal.select('test'));

    console.log("closing mem", await surreal.close());
}

async function run_skv() {
    const surreal = new Surreal({
        engines: surrealdbNodeEngines()
    });

    console.log("connecting skv", await surreal.connect("surrealkv://test.skv", { versionCheck: false }));

    console.log("using skv", await surreal.use({ namespace: "test", database: "test" }));

    console.log("creating skv", await surreal.create('test', { val: 42 }));

    console.log("selecting skv", await surreal.select('test'));

    console.log("closing skv", await surreal.close());
}

run_mem()
run_skv()