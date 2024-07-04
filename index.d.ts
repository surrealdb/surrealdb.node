import { ConnectionOptions } from "./lib-src/embedded.js";

export declare class SurrealdbNodeEngine {
  execute(data: Uint8Array): Promise<Uint8Array>
  static connect(endpoint: string, opts?: ConnectionOptions): Promise<SurrealdbNodeEngine>
  free(): Promise<void>
  static version(): string
}
