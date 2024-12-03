// Definitions for Plugins
pub mod plugins {
    // The trait that all plugins will implement.
    // It should support the agent and server system;
    // for ease of development, although this means the
    // lib will contain some server functionality (opsec risk).
    pub trait Plugin:
        serde::Serialize + for<'de> serde::Deserialize<'de> + Send + Sync + 'static
    {
        type PluginMessage: PluginMessage;
        type SystemMessage: SystemMessage;

        // This function should be implemented to handle the keys within the implemented plugin structures.
        //fn handle_keys(&self) -> super::core::Result<()>;

        fn generate_heartbeat(&self) -> super::core::Result<impl PluginMessage>;

        fn agent_runtime(
            &self,
            message: Self::PluginMessage,
        ) -> super::core::Result<Option<impl PluginMessage>>;

        fn server_runtime(
            &self,
            message: Self::PluginMessage,
        ) -> super::core::Result<impl PluginMessage>;

        /// Handles the command pooling infrastrucsure related to the RWLock
        fn pool_handler(
            &mut self,
            message: Self::SystemMessage,
        ) -> super::core::Result<impl SystemMessage>;

        /// Allows for the server to check the plugin for shutdown initialization call
        fn shutdown_check(&self) -> super::core::Result<bool>;
    }

    pub trait PluginMessage: std::any::Any {
        // Method to serialize the message
        fn serialize(&self) -> super::core::Result<Vec<u8>>;

        // Method to deserialize the message from bytes
        fn deserialize(bytes: Vec<u8>) -> super::core::Result<Self>
        where
            Self: Sized;
    }

    // Define a trait for messages that exchanged between the CLI and Server backend, generically.
    pub trait SystemMessage: std::any::Any + Send + Sync + 'static {
        fn serialize(&self) -> super::core::Result<Vec<u8>>;

        fn deserialize(bytes: Vec<u8>) -> super::core::Result<Self>
        where
            Self: Sized;

        fn generate(request: super::cli::Request) -> super::core::Result<Self>
        where
            Self: Sized;
    }

    // Downcasting PluginMessage Trait, defining the as_any function used to cast down.
    pub trait AsAnyPluginMessage {
        fn as_any(&self) -> &dyn std::any::Any;
    }

    // Implementation for Generic Type, so long as they implemented PluginMessage
    impl<T: 'static + PluginMessage> AsAnyPluginMessage for T {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // Downcasting SystemMessage Trait, defining the as_any function used to cast down.
    pub trait AsAnySystemMessage {
        fn as_any(&self) -> &dyn std::any::Any;
    }

    // Implementation for Generic Type, so long as they implemented SystemMessage
    impl<T: 'static + SystemMessage> AsAnySystemMessage for T {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

// Generic Component Utilities
pub mod core {
    // // Useful type for wrapping results, with boxed errors that implement standard error...
    // pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

    // Generic Result Typing Used Everywhere, To Keep Things Thread Safe And Generic
    type Error = Box<dyn std::error::Error + Send + Sync>;
    pub type Result<T> = std::result::Result<T, Error>;

    // Function to generically downcast plugin messages (agent cross server) between implemented traits.
    pub fn downcast_plugin_message<T, E>(message: &T) -> Result<&E>
    where
        T: crate::plugins::AsAnyPluginMessage + crate::plugins::PluginMessage,
        E: 'static,
    {
        if let Some(downcasted_message) = message.as_any().downcast_ref::<E>() {
            return Ok(downcasted_message);
        } else {
            Err("This message doesn't implement PluginMessage...".into())
        }
    }

    // Function to generically downcast system messages (cli cross server) between implemented traits.
    pub fn downcast_system_message<T, E>(message: &T) -> Result<&E>
    where
        T: crate::plugins::AsAnySystemMessage + crate::plugins::SystemMessage,
        E: 'static,
    {
        if let Some(downcasted_message) = message.as_any().downcast_ref::<E>() {
            return Ok(downcasted_message);
        } else {
            Err("This message doesn't implement SystemMessage...".into())
        }
    }
}

pub mod cryptography {

    use orion::{aead, kex};

    use crate::plugins::Plugin;

    pub fn generate_keys(plugin: impl Plugin) {
        // let session_server = kex::EphemeralClientSession::new()?;
        // plugin.
        //
        todo!()
    }

    // Takes in server public key and the received session ID, should attach the shared keys to the Plugin state.
    pub fn client_kex_handler(
        agency_key: Vec<u8>,
        session_id: Vec<u8>,
    ) -> super::core::Result<(kex::PublicKey, String)> {
        // let public_key_slice: &[u8] = agency_key.as_slice();
        // let server_public_key = kex::PublicKey::from_slice(public_key_slice)?;
        // Ok((server_public_key, String::from_utf8(serialized_session_id)?))
        todo!()
    }

    // Takes in Client Key as arguments and returns server public key, and session ID for client
    pub fn server_kex_handler(
        agency_key: Vec<u8>,
    ) -> super::core::Result<(kex::SessionKeys, String)> {
        // let session_server = kex::EphemeralClientSession::new()?;
        // let client_public_key = session_server.public_key();

        // //using the public value, generate the secrets.
        // let session_keys = session_server.establish_with_server(&server_public_key)?;
        // Ok((session_keys, session_id))
        todo!()
    }

    pub async fn encrypt(
        tx_key: &kex::SecretKey,
        message: Vec<u8>,
    ) -> super::core::Result<Vec<u8>> {
        let buffer = aead::seal(tx_key, &message)?;
        return Ok(buffer);
    }

    pub async fn decrypt(
        rx_key: &kex::SecretKey,
        message: Vec<u8>,
    ) -> super::core::Result<Vec<u8>> {
        let buffer = aead::open(rx_key, &message)?;
        return Ok(buffer);
    }
}

pub mod cli {

    use std::{fs::File, io::Read};

    use std::{hash::Hash, io::Write};

    use clap::{builder::ValueParser, error::ErrorKind};

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub enum Request {
        QUEUE(Queue),
        DEQUEUE(Dequeue),
        LOCK(Lock),
        LIST(List),
        QUIT(Quit),
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Response {
        pub confirmation: String,
        pub commands: Vec<String>,
        pub targets: Vec<AirportCodes>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Queue {
        pub command: String,
        pub targets: Vec<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Dequeue {
        pub command: String,
        pub targets: Vec<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Lock {
        pub targets: Vec<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct List {
        pub targets: Vec<String>,
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Quit {
        pub destroy: bool,
    }

    #[derive(
        Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
    )]
    pub enum AirportCodes {
        /// All Airports
        ALL {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// John F. Kennedy International Airport (New York, USA)
        JFK {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// London Heathrow Airport (London, UK)
        LHR {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Charles de Gaulle Airport (Paris, France)
        CDG {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Beijing Capital International Airport (Beijing, China)
        PEK {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// San Francisco International Airport (San Francisco, USA)
        SFO {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Sydney Kingsford Smith Airport (Sydney, Australia)
        SYD {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Haneda Airport (Tokyo, Japan)
        HND {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Dubai International Airport (Dubai, UAE)
        DXB {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Toronto Pearson International Airport (Toronto, Canada)
        YYZ {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Munich Airport (Munich, Germany)
        MUC {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Amsterdam Airport Schiphol (Amsterdam, Netherlands)
        AMS {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Incheon International Airport (Seoul, South Korea)
        ICN {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Indira Gandhi International Airport (New Delhi, India)
        DEL {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Miami International Airport (Miami, USA)
        MIA {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Singapore Changi Airport (Singapore)
        SIN {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Kuala Lumpur International Airport (Kuala Lumpur, Malaysia)
        KUL {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Istanbul Airport (Istanbul, Turkey)
        IST {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Mexico City International Airport (Mexico City, Mexico)
        MEX {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Syracuse Hancock International Airport (Syracuse, USA)
        SYR {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
        /// Chhatrapati Shivaji Maharaj International Airport (Mumbai, India)
        BOM {
            location: String,
            target_number: u32,
            target_name: String,
            locked: bool,
        },
    }

    /// REPL line reading utility
    pub async fn read_line() -> super::core::Result<String> {
        write!(std::io::stdout(), "\n>> ").map_err(|error| error.to_string())?;
        std::io::stdout()
            .flush()
            .map_err(|error| error.to_string())?;
        let mut buffer = String::new();
        std::io::stdin()
            .read_line(&mut buffer)
            .map_err(|error| error.to_string())?;
        Ok(buffer)
    }

    /// Displaying ansi banner utility
    pub fn banner() -> super::core::Result<()> {
        // Open the .ans file for reading
        let file_path = "banner.ans"; // Replace with your file path
        let mut file = File::open(file_path)?;

        // Read the contents of the file into a buffer
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Convert the buffer to a string
        let contents = String::from_utf8_lossy(&buffer);

        // Print the contents to the terminal
        print!("{}", contents);

        // Return result
        Ok(())
    }

    /// Regex validation wrapper for complex inputs in clap cli.
    pub fn regex_validator(pattern_string: &'static str) -> ValueParser {
        ValueParser::from(
            move |input: &str| -> std::result::Result<String, clap::Error> {
                let regex = regex::Regex::new(pattern_string).unwrap();
                match regex.is_match(input) {
                    true => Ok(input.to_owned()),
                    false => Err(clap::Error::new(ErrorKind::ValueValidation)),
                }
            },
        )
    }

    /// Regex to match valid ports.
    pub fn port_validator() -> ValueParser {
        return regex_validator(
            r"^([1-9][0-9]{0,3}|[1-5][0-9]{4}|6[0-4][0-9]{3}|65[0-4][0-9]{2}|655[0-2][0-9]|6553[0-5])$",
        );
    }

    pub fn record_validator() -> ValueParser {
        return regex_validator(r"");
    }

    /// Regex to match any valid IPv4 Address with cidr codes congruent to mod 8 up to and including 32.
    pub fn is_ipv4_cidr() -> ValueParser {
        let pattern_string = r"^((25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)\.){3}(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)($|/(0|8|16|24|32))?$";
        return regex_validator(pattern_string);
    }
}
