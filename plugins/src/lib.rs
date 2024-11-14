extern crate utilities;

/*
    This is the Microchip Plugin, code here was made/inspired by, or in part, by myrrha, mineo333, and chasek
    This implements the frida gum based process injection agent functionality, using dyn-sections and code cave generation.
    Open License GPLv3 2024
*/

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MicrochipPlugin {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct MicrochipMessage {
    pub shellcode: Vec<u8>,
    pub source: String,
    pub destination: String,
    pub agency_key: Vec<u8>,
}

pub struct TempestariiMessage;

impl utilities::plugins::PluginMessage for MicrochipMessage {
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

impl std::fmt::Display for MicrochipMessage {
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
            "MicrochipMessage:\n\
            \tSource: {}\n\
            \tDestination: {}\n\
            \tShellcode (hex): {}\n\
            \tAgency Key (hex): {}",
            self.source, self.destination, shellcode_hex, agency_key_hex
        )
    }
}

impl utilities::plugins::SystemMessage for TempestarriMessage {
    fn parse(input: &str) -> Self
    where
        Self: Sized,
    {
        todo!()
    }
}

impl utilities::plugins::Plugin for MicrochipPlugin {
    type PluginMessage = MicrochipMessage;

    type SystemMessage = TempestariiMessage;

    fn generate_heartbeat(
        &self,
    ) -> utilities::core::Result<impl utilities::plugins::PluginMessage> {
        let client_message = MicrochipMessage {
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
        utilities::plugins::utility_function();

        if server_message.agency_key == vec![1, 2, 3] {
            let client_message = MicrochipMessage {
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
        utilities::plugins::utility_function();

        if client_message.destination == String::from("Tempestarii") {
            let server_response = MicrochipMessage {
                shellcode: Vec::new(),
                source: String::from("Server 1"),
                destination: String::from("Agent 1"),
                agency_key: vec![1, 2, 3],
            };

            return Ok(server_response);
        } else {
            let server_response = MicrochipMessage {
                shellcode: Vec::new(),
                source: String::from("Server 1"),
                destination: String::from("Agent 1"),
                agency_key: Vec::new(),
            };

            return Ok(server_response);
        }
    }
}
