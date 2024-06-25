import Surreal from "surrealdb.js";
import { surrealdbNodeEngines } from "./lib-src/embedded.ts";

async function run_mem() {
    const surreal = new Surreal({
        engines: surrealdbNodeEngines()
    });

    console.log("connecting", await surreal.connect("mem://", { versionCheck: false }));

    console.log("using", await surreal.use({ namespace: "test", database: "test" }));

    console.log("listening", await surreal.live("test", (res) => console.log("recieved live" + res)));

    console.log("creating", await surreal.create('test', { val: 42 }));

    console.log("selecting", await surreal.select('test'));

    console.log("closing", await surreal.close());
}

run_mem()