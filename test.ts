import Surreal from "surrealdb";
import { surrealdbNodeEngines } from "./lib-src/embedded.ts";

async function run(endpoint) {
    const surreal = new Surreal({
        engines: surrealdbNodeEngines(),
    });

    console.log(
        "connecting " + endpoint,
        await surreal.connect(endpoint, { versionCheck: false }),
    );

    console.log(
        "using " + endpoint,
        await surreal.use({ namespace: "test", database: "test" }),
    );

	console.log("version " + await surreal.version());

    console.log(
        "creating " + endpoint,
        await surreal.create("test", { val: 42 }),
    );

    console.log("selecting " + endpoint, await surreal.select("test"));

    console.log("exporting " + endpoint, await surreal.export());

    console.log("closing " + endpoint, await surreal.close());
}

run("surrealkv://test.skv");
run("mem://");
