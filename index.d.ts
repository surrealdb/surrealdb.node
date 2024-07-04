import { ConnectionOptions } from "./lib-src/embedded.js";
import { ConnectionOptions } from "./lib-src/embedded.js";
/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export declare class SurrealdbNodeEngine {
    execute(data: Uint8Array): Promise<Uint8Array>;
    static connect(
        endpoint: string,
        opts?: ConnectionOptions,
    ): Promise<SurrealdbNodeEngine>;
    free(): Promise<void>;
    static version(): string;
}
