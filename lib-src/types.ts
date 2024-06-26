type CapabilitiesAllowDenyList = {
    allow?: boolean | string[];
    deny?: boolean | string[];
};

type ConnectionOptions = {
    strict?: boolean;
    query_timeout?: number;
    transaction_timeout?: number;
    capabilities?: boolean | {
        guest_access?: boolean;
        functions?: boolean | string[] | CapabilitiesAllowDenyList;
        network_targets?: boolean | string[] | CapabilitiesAllowDenyList;
    }
}

export type { ConnectionOptions };