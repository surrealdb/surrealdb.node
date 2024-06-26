use napi::Error;
use serde::Deserialize;
use std::{collections::HashSet, hash::Hash, str::FromStr};
use surrealdb_core::dbs::capabilities;

use crate::error::err_map;

#[derive(Deserialize)]
pub struct Options {
    pub strict: Option<bool>,
    pub query_timeout: Option<u8>,
    pub transaction_timeout: Option<u8>,
    pub capabilities: Option<CapabilitiesConfig>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum CapabilitiesConfig {
    Bool(bool),
    Capabilities {
        scripting: Option<bool>,
        guest_access: Option<bool>,
        live_query_notifications: Option<bool>,
        functions: Option<Targets>,
        network_targets: Option<Targets>,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Targets {
    Bool(bool),
    Array(HashSet<String>),
    Config {
        allow: Option<TargetsConfig>,
        deny: Option<TargetsConfig>,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum TargetsConfig {
    Bool(bool),
    Array(HashSet<String>),
}

// macro_rules! process_targets {
//     ($set:ident) => {{
//         let mut functions = HashSet::with_capacity($set.len());
//         for function in $set {
//             functions.insert(function.parse()?);
//         }
//         capabilities::Targets::Some(functions)
//     }};
// }

trait TargetExt<T: Hash + Eq + FromStr> {
    fn process(&self) -> Result<capabilities::Targets<T>, napi::Error>;
}

impl<T> TargetExt<T> for HashSet<String>
where
    T: Hash + Eq + FromStr,
    <T as FromStr>::Err: std::error::Error,
{
    fn process(&self) -> Result<capabilities::Targets<T>, napi::Error> {
        let mut acc = HashSet::with_capacity(self.len());
        for item in self {
            acc.insert(item.parse().map_err(err_map)?);
        }
        Ok(capabilities::Targets::Some(acc))
    }
}

impl TryFrom<CapabilitiesConfig> for surrealdb::dbs::Capabilities {
    type Error = Error;

    fn try_from(config: CapabilitiesConfig) -> Result<Self, Error> {
        match config {
            CapabilitiesConfig::Bool(true) => Ok(Self::all()),
            CapabilitiesConfig::Bool(false) => {
                Ok(Self::default().with_functions(capabilities::Targets::None))
            }
            CapabilitiesConfig::Capabilities {
                scripting,
                guest_access,
                live_query_notifications,
                functions,
                network_targets,
            } => {
                let mut capabilities = Self::default();

                if let Some(scripting) = scripting {
                    capabilities = capabilities.with_scripting(scripting);
                }

                if let Some(guest_access) = guest_access {
                    capabilities = capabilities.with_guest_access(guest_access);
                }

                if let Some(live_query_notifications) = live_query_notifications {
                    capabilities =
                        capabilities.with_live_query_notifications(live_query_notifications);
                }

                if let Some(functions) = functions {
                    match functions {
                        Targets::Bool(functions) => match functions {
                            true => {
                                capabilities =
                                    capabilities.with_functions(capabilities::Targets::All);
                            }
                            false => {
                                capabilities =
                                    capabilities.with_functions(capabilities::Targets::None);
                            }
                        },
                        Targets::Array(set) => {
                            capabilities = capabilities.with_functions(set.process()?);
                        }
                        Targets::Config { allow, deny } => {
                            if let Some(config) = allow {
                                match config {
                                    TargetsConfig::Bool(functions) => match functions {
                                        true => {
                                            capabilities = capabilities
                                                .with_functions(capabilities::Targets::All);
                                        }
                                        false => {
                                            capabilities = capabilities
                                                .with_functions(capabilities::Targets::None);
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        capabilities = capabilities.with_functions(set.process()?);
                                    }
                                }
                            }

                            if let Some(config) = deny {
                                match config {
                                    TargetsConfig::Bool(functions) => match functions {
                                        true => {
                                            capabilities = capabilities
                                                .without_functions(capabilities::Targets::All);
                                        }
                                        false => {
                                            capabilities = capabilities
                                                .without_functions(capabilities::Targets::None);
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        capabilities =
                                            capabilities.without_functions(set.process()?);
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(network_targets) = network_targets {
                    match network_targets {
                        Targets::Bool(network_targets) => match network_targets {
                            true => {
                                capabilities =
                                    capabilities.with_network_targets(capabilities::Targets::All);
                            }
                            false => {
                                capabilities =
                                    capabilities.with_network_targets(capabilities::Targets::None);
                            }
                        },
                        Targets::Array(set) => {
                            capabilities = capabilities.with_network_targets(set.process()?);
                        }
                        Targets::Config { allow, deny } => {
                            if let Some(config) = allow {
                                match config {
                                    TargetsConfig::Bool(network_targets) => match network_targets {
                                        true => {
                                            capabilities = capabilities
                                                .with_network_targets(capabilities::Targets::All);
                                        }
                                        false => {
                                            capabilities = capabilities
                                                .with_network_targets(capabilities::Targets::None);
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        capabilities =
                                            capabilities.with_network_targets(set.process()?);
                                    }
                                }
                            }

                            if let Some(config) = deny {
                                match config {
                                    TargetsConfig::Bool(network_targets) => match network_targets {
                                        true => {
                                            capabilities = capabilities.without_network_targets(
                                                capabilities::Targets::All,
                                            );
                                        }
                                        false => {
                                            capabilities = capabilities.without_network_targets(
                                                capabilities::Targets::None,
                                            );
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        capabilities =
                                            capabilities.without_network_targets(set.process()?);
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(capabilities)
            }
        }
    }
}

impl TryFrom<CapabilitiesConfig> for surrealdb::opt::capabilities::Capabilities {
    type Error = Error;

    fn try_from(config: CapabilitiesConfig) -> Result<Self, Self::Error> {
        match config {
            CapabilitiesConfig::Bool(true) => Ok(Self::all()),
            CapabilitiesConfig::Bool(false) => Ok(Self::none()),
            CapabilitiesConfig::Capabilities {
                scripting,
                guest_access,
                live_query_notifications,
                functions,
                network_targets,
            } => {
                let mut capabilities = Self::default();

                if let Some(scripting) = scripting {
                    capabilities = capabilities.with_scripting(scripting);
                }

                if let Some(guest_access) = guest_access {
                    capabilities = capabilities.with_guest_access(guest_access);
                }

                if let Some(live_query_notifications) = live_query_notifications {
                    capabilities =
                        capabilities.with_live_query_notifications(live_query_notifications);
                }

                if let Some(functions) = functions {
                    match functions {
                        Targets::Bool(functions) => match functions {
                            true => {
                                capabilities = capabilities.with_allow_all_functions();
                            }
                            false => {
                                capabilities = capabilities.with_allow_none_functions();
                            }
                        },
                        Targets::Array(set) => {
                            for func in set {
                                capabilities =
                                    capabilities.with_allow_function(&func).map_err(err_map)?;
                            }
                        }
                        Targets::Config { allow, deny } => {
                            if let Some(config) = allow {
                                match config {
                                    TargetsConfig::Bool(functions) => match functions {
                                        true => {
                                            capabilities = capabilities.with_allow_all_functions();
                                        }
                                        false => {
                                            capabilities = capabilities.with_allow_none_functions();
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        for func in set {
                                            capabilities = capabilities
                                                .with_allow_function(&func)
                                                .map_err(err_map)?;
                                        }
                                    }
                                }
                            }

                            if let Some(config) = deny {
                                match config {
                                    TargetsConfig::Bool(functions) => match functions {
                                        true => {
                                            capabilities = capabilities.with_allow_all_functions();
                                        }
                                        false => {
                                            capabilities = capabilities.with_allow_none_functions();
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        for func in set {
                                            capabilities = capabilities
                                                .with_allow_function(&func)
                                                .map_err(err_map)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(network_targets) = network_targets {
                    match network_targets {
                        Targets::Bool(network_targets) => match network_targets {
                            true => {
                                capabilities = capabilities.with_allow_all_net_targets();
                            }
                            false => {
                                capabilities = capabilities.with_allow_none_net_targets();
                            }
                        },
                        Targets::Array(set) => {
                            for net in set {
                                capabilities =
                                    capabilities.with_allow_net_target(&net).map_err(err_map)?;
                            }
                        }
                        Targets::Config { allow, deny } => {
                            if let Some(config) = allow {
                                match config {
                                    TargetsConfig::Bool(network_targets) => match network_targets {
                                        true => {
                                            capabilities =
                                                capabilities.with_allow_all_net_targets();
                                        }
                                        false => {
                                            capabilities = capabilities.with_deny_none_net_target();
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        for net in set {
                                            capabilities = capabilities
                                                .with_allow_net_target(&net)
                                                .map_err(err_map)?;
                                        }
                                    }
                                }
                            }

                            if let Some(config) = deny {
                                match config {
                                    TargetsConfig::Bool(network_targets) => match network_targets {
                                        true => {
                                            capabilities = capabilities.with_deny_all_function();
                                        }
                                        false => {
                                            capabilities = capabilities.with_deny_none_function();
                                        }
                                    },
                                    TargetsConfig::Array(set) => {
                                        for net in set {
                                            capabilities = capabilities
                                                .with_allow_net_target(&net)
                                                .map_err(err_map)?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(capabilities)
            }
        }
    }
}
