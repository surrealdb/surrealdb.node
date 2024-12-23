import { SurrealdbNodeEngine as Sne } from "../index.js";

import {
    AbstractEngine,
    ConnectionStatus,
    ConnectionUnavailable,
    type EngineEvents,
    Engines,
    ExportOptions,
    getIncrementalID,
    type RpcRequest,
    type RpcResponse,
    UnexpectedConnectionError,
} from "surrealdb";

/**
 * Construct the engines for the SurrealDB Nodejs implementation. This
 * includes support for `mem` and `surrealkv` protocols.
 * 
 * @param opts Configuration options
 * @returns The engines
 */
export function surrealdbNodeEngines(opts?: ConnectionOptions): Engines {

    class NodeEmbeddedEngine extends AbstractEngine {

        ready: Promise<void> | undefined = undefined;
        reader?: Promise<void>;
        db?: Sne;

        async version(): Promise<string> {
            return Sne.version();
        }

        setStatus<T extends ConnectionStatus>(
            status: T,
            ...args: EngineEvents[T]
        ) {
            this.status = status;
            this.emitter.emit(status, args);
        }

        async connect(url: URL) {
            this.connection.url = url;
            this.setStatus(ConnectionStatus.Connecting);
            const ready = (async (resolve, reject) => {
                const db = await Sne.connect(url.toString(), opts).catch(
                    (e) => {
                        console.log(e);
                        const error = new UnexpectedConnectionError(
                            typeof e === "string"
                                ? e
                                : "error" in e
                                  ? e.error
                                  : "An unexpected error occurred",
                        );
                        this.setStatus(ConnectionStatus.Error, error);
                        throw e;
                    },
                );

                this.db = db;
                this.setStatus(ConnectionStatus.Connected);

                // this.reader = (async () => {
                // 	const reader = db.notifications().getReader();
                // 	while (this.connected) {
                // 		const { done, value } = await reader.read();
                // 		if (done) break;
                // 		const raw = value as Uint8Array;
                // 		const { id, action, result } = this.decodeCbor(raw.buffer);
                // 		if (id) this.emitter.emit(`live-${id.toString()}`, [action, result], true);
                // 	}
                // })();
            })();

            this.ready = ready;
            return await ready;
        }

        async disconnect(): Promise<void> {
            this.connection = {
                url: undefined,
                namespace: undefined,
                database: undefined,
                token: undefined,
            };

            await this.ready;
            this.ready = undefined;
            this.db?.free();
            this.db = undefined;
            await this.reader;
            this.reader = undefined;
			
            if (this.status !== ConnectionStatus.Disconnected) {
                this.setStatus(ConnectionStatus.Disconnected);
            }
        }

        async rpc<
            Method extends string,
            Params extends unknown[] | undefined,
            Result,
        >(request: RpcRequest<Method, Params>): Promise<RpcResponse<Result>> {
            await this.ready;
            if (!this.db) throw new ConnectionUnavailable();

            // It's not realistic for the message to ever arrive before the listener is registered on the emitter
            // And we don't want to collect the response messages in the emitter
            // So to be sure we simply subscribe before we send the message :)

            const id = getIncrementalID();
            const res: RpcResponse = await this.db
                .execute(new Uint8Array(this.encodeCbor({ id, ...request })))
                .then((raw) => ({ result: this.decodeCbor(raw.buffer) }))
                .catch((message) => ({ error: { code: -1, message } }));

            if ("result" in res) {
                switch (request.method) {
                    case "use": {
                        this.connection.namespace = request
                            .params?.[0] as string;
                        this.connection.database = request
                            .params?.[1] as string;
                        break;
                    }

                    case "signin":
                    case "signup": {
                        this.connection.token = res.result as string;
                        break;
                    }

                    case "authenticate": {
                        this.connection.token = request.params?.[0] as string;
                        break;
                    }

                    case "invalidate": {
                        this.connection.token = undefined;
                        break;
                    }
                }
            }

            return res as RpcResponse<Result>;
        }

        get connected() {
            return !!this.db;
        }

		export(options?: Partial<ExportOptions>): Promise<string> {
			return this.db.export(options ? new Uint8Array(this.encodeCbor(options)) : undefined);
		}
		
    }

    return {
        mem: NodeEmbeddedEngine,
        surrealkv: NodeEmbeddedEngine,
       "surrealkv+versioned": NodeEmbeddedEngine,
    };
}

export type CapabilitiesAllowDenyList = {
    allow?: boolean | string[];
    deny?: boolean | string[];
};

export type ConnectionOptions = {
    strict?: boolean;
    query_timeout?: number;
    transaction_timeout?: number;
    capabilities?:
        | boolean
        | {
              guest_access?: boolean;
              functions?: boolean | string[] | CapabilitiesAllowDenyList;
              network_targets?: boolean | string[] | CapabilitiesAllowDenyList;
          };
};
