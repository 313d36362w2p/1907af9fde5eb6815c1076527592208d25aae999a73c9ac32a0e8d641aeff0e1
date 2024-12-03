extern crate utilities;

/*
    This is the Microchip Plugin, code here was made/inspired by, or in part, by myrrha, mineo333, nerelod, and chasek
    This implements the frida gum based process injection agent functionality, using dyn-sections and code cave generation.
    Open License GPLv3 2024
*/

pub mod local_utilities {
    pub fn filter_commands_by_target(
        command_pool: &[super::MicrochipCommand],
        target: String,
    ) -> utilities::core::Result<Vec<String>> {
        Ok(command_pool
            .iter() // Iterate over the command pool
            .filter(|command| command.targets.contains(&target.to_string())) // Filter commands that contain the target
            .map(|command| command.command.clone()) // Map to just the command string
            .collect()) // Collect into a vector of command strings
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicrochipPlugin {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub command_pool: Vec<MicrochipCommand>,
    pub shutdown: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicrochipCommand {
    pub targets: Vec<String>,
    pub command: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicrochipPluginMessage {
    pub shellcode: Vec<u8>,
    pub source: String,
    pub destination: String,
    pub agency_key: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MicrochipSystemMessage {
    pub ip: String,
    pub request: Option<utilities::cli::Request>,
    pub response: Option<utilities::cli::Response>,
}

impl utilities::plugins::PluginMessage for MicrochipPluginMessage {
    fn serialize(&self) -> utilities::core::Result<Vec<u8>> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json.as_bytes().to_vec()),
            Err(error) => {
                eprintln!("Failed to serialize the struct: {}", error);
                Err(error.to_string().into())
            }
        }
    }

    fn deserialize(bytes: Vec<u8>) -> utilities::core::Result<Self> {
        let deserialized_struct: Self = serde_json::from_slice(&bytes)?;
        Ok(deserialized_struct)
    }
}

impl std::fmt::Display for MicrochipPluginMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let shellcode_hex: String = self
            .shellcode
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        let agency_key_hex: String = self
            .agency_key
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect();
        write!(
            f,
            "MicrochipPluginMessage:\n\
            \tSource: {}\n\
            \tDestination: {}\n\
            \tShellcode (hex): {}\n\
            \tAgency Key (hex): {}",
            self.source, self.destination, shellcode_hex, agency_key_hex
        )
    }
}

impl utilities::plugins::SystemMessage for MicrochipSystemMessage {
    fn serialize(&self) -> utilities::core::Result<Vec<u8>> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json.as_bytes().to_vec()),
            Err(error) => {
                eprintln!("Failed to serialize the struct: {}", error);
                Err(error.to_string().into())
            }
        }
    }

    fn deserialize(bytes: Vec<u8>) -> utilities::core::Result<Self> {
        let deserialized_struct: Self = serde_json::from_slice(&bytes)?;
        Ok(deserialized_struct)
    }

    fn generate(request: utilities::cli::Request) -> utilities::core::Result<Self> {
        let packet = MicrochipSystemMessage {
            ip: String::from("Tempestarii"),
            request: Some(request),
            response: None,
        };

        Ok(packet)
    }
}

impl std::fmt::Display for MicrochipSystemMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "MicrochipSystemMessage:\n\
            \tIP: {}\n\
            \tRequest: {}\n\
            \tResponse: {}",
            self.ip,
            self.request
                .as_ref()
                .map_or("None".to_string(), |r| format!("{:?}", r)),
            self.response
                .as_ref()
                .map_or("None".to_string(), |r| format!("{:?}", r))
        )
    }
}

impl utilities::plugins::Plugin for MicrochipPlugin {
    type PluginMessage = MicrochipPluginMessage;

    type SystemMessage = MicrochipSystemMessage;

    fn generate_heartbeat(
        &self,
    ) -> utilities::core::Result<impl utilities::plugins::PluginMessage> {
        let client_message = MicrochipPluginMessage {
            shellcode: Vec::new(),
            source: String::from(""),
            destination: String::from("Tempestarii"),
            agency_key: Vec::new(),
        };

        return Ok(client_message);
    }

    fn agent_runtime(
        &self,
        server_message: Self::PluginMessage,
    ) -> utilities::core::Result<Option<impl utilities::plugins::PluginMessage>> {
        if server_message.agency_key == vec![1, 2, 3] {
            let client_message = MicrochipPluginMessage {
                shellcode: Vec::new(),
                source: String::from("Agent 1"),
                destination: String::from("Server 1"),
                agency_key: vec![3, 2, 1],
            };

            return Ok(Some(client_message));
        } else {
            return Ok(None);
        }
    }

    fn server_runtime(
        &self,
        client_message: Self::PluginMessage,
    ) -> utilities::core::Result<impl utilities::plugins::PluginMessage> {
        if client_message.destination == String::from("Tempestarii") {
            let server_response = MicrochipPluginMessage {
                shellcode: Vec::new(),
                source: String::from("Server 1"),
                destination: String::from("Agent 1"),
                agency_key: vec![1, 2, 3],
            };

            return Ok(server_response);
        } else {
            let server_response = MicrochipPluginMessage {
                shellcode: Vec::new(),
                source: String::from("Server 1"),
                destination: String::from("Agent 1"),
                agency_key: Vec::new(),
            };

            return Ok(server_response);
        }
    }

    fn pool_handler(
        &mut self,
        message: Self::SystemMessage,
    ) -> utilities::core::Result<impl utilities::plugins::SystemMessage> {
        match message.request {
            Some(request) => match request {
                utilities::cli::Request::QUEUE(state) => {
                    let response = MicrochipSystemMessage {
                        ip: String::from("Storm"),
                        request: None,
                        response: Some(utilities::cli::Response {
                            confirmation: String::from("Command Registered Successfully"),
                            commands: vec![state.command],
                            targets: vec![utilities::cli::AirportCodes::ALL {
                                location: String::from("Nowhere Good"),
                                target_number: 1,
                                target_name: String::from("Chase K's House"),
                                locked: false,
                            }],
                        }),
                    };
                    Ok(response)
                }
                utilities::cli::Request::DEQUEUE(state) => {
                    let response = MicrochipSystemMessage {
                        ip: String::from("Storm"),
                        request: None,
                        response: Some(utilities::cli::Response {
                            confirmation: String::from("Command Unregistered Successfully"),
                            commands: vec![state.command],
                            targets: vec![utilities::cli::AirportCodes::ALL {
                                location: String::from("Nowhere Good"),
                                target_number: 1,
                                target_name: String::from("Chase K's House"),
                                locked: false,
                            }],
                        }),
                    };
                    Ok(response)
                }
                utilities::cli::Request::LOCK(_state) => {
                    // Lock isn't going to do much for right now
                    let response = MicrochipSystemMessage {
                        ip: String::from("Storm"),
                        request: None,
                        response: Some(utilities::cli::Response {
                            confirmation: String::from("Target Status Locked"),
                            commands: vec![],
                            targets: vec![utilities::cli::AirportCodes::ALL {
                                location: String::from("Nowhere Good"),
                                target_number: 1,
                                target_name: String::from("Chase K's House"),
                                locked: false,
                            }],
                        }),
                    };
                    Ok(response)
                }
                utilities::cli::Request::LIST(state) => {
                    let mut commands = vec![];

                    for target in state.targets {
                        let filtered_commands =
                            local_utilities::filter_commands_by_target(&self.command_pool, target)?;
                        commands.extend(filtered_commands);
                    }

                    let response = MicrochipSystemMessage {
                        ip: String::from("Storm"),
                        request: None,
                        response: Some(utilities::cli::Response {
                            confirmation: String::from("Request For Targeted Commands Successful"),
                            commands,
                            targets: vec![utilities::cli::AirportCodes::ALL {
                                location: String::from("Nowhere Good"),
                                target_number: 1,
                                target_name: String::from("Chase K's House"),
                                locked: false,
                            }],
                        }),
                    };
                    Ok(response)
                }
                utilities::cli::Request::QUIT(state) => {
                    self.shutdown = true;
                    if state.destroy {
                        let response = MicrochipSystemMessage {
                            ip: String::from("Storm"),
                            request: None,
                            response: Some(utilities::cli::Response {
                                confirmation: String::from(
                                    "Triggering Massive Destruction of Infrastructure",
                                ),
                                commands: vec![],
                                targets: vec![utilities::cli::AirportCodes::ALL {
                                    location: String::from("Nowhere Good"),
                                    target_number: 1,
                                    target_name: String::from("Chase K's House"),
                                    locked: false,
                                }],
                            }),
                        };
                        Ok(response)
                    } else {
                        let response = MicrochipSystemMessage {
                            ip: String::from("Storm"),
                            request: None,
                            response: Some(utilities::cli::Response {
                                confirmation: String::from("Triggering Quiet Closure"),
                                commands: vec![],
                                targets: vec![utilities::cli::AirportCodes::ALL {
                                    location: String::from("Nowhere Good"),
                                    target_number: 1,
                                    target_name: String::from("Chase K's House"),
                                    locked: false,
                                }],
                            }),
                        };
                        Ok(response)
                    }
                }
            },
            None => {
                let response = MicrochipSystemMessage {
                    ip: String::from("Storm"),
                    request: None,
                    response: None,
                };
                Ok(response)
            }
        }
    }

    fn shutdown_check(&self) -> utilities::core::Result<bool> {
        Ok(self.shutdown)
    }
}
