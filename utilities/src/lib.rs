// Definitions for Plugins
pub mod plugins {
    // The trait that all plugins will implement.
    // It should support the agent and server system;
    // for ease of development, although this means the
    // lib will contain some server functionality (opsec risk).
    pub trait Plugin: serde::Serialize + for<'de> serde::Deserialize<'de> {
        type PluginMessage: PluginMessage;
        type SystemMessage: SystemMessage;

        fn generate_heartbeat(&self) -> super::core::Result<impl PluginMessage>;
        fn agent_runtime(
            &self,
            message: Self::PluginMessage,
        ) -> super::core::Result<Option<impl PluginMessage>>;
        fn server_runtime(
            &self,
            message: Self::PluginMessage,
        ) -> super::core::Result<impl PluginMessage>;
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
    pub trait SystemMessage {
        fn parse(input: &str) -> Self
        where
            Self: Sized;
    }

    // Downcasting Trait, defining the as_any function used to cast down.
    pub trait AsAny {
        fn as_any(&self) -> &dyn std::any::Any;
    }

    // Implementation for Generic Type, so long as they implemented PluginMessage
    impl<T: 'static + PluginMessage> AsAny for T {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
}

// Generic Component Utilities
pub mod core {
    // Useful type for wrapping results, with boxed errors that implement standard error...
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

    // Function to generically downcast between implemented traits.
    pub fn downcast_message<T, E>(message: &T) -> Result<&E>
    where
        T: crate::plugins::AsAny + crate::plugins::PluginMessage,
        E: 'static,
    {
        if let Some(downcasted_message) = message.as_any().downcast_ref::<E>() {
            return Ok(downcasted_message);
        } else {
            Err("This message doesn't implement PluginMessage...".into())
        }
    }
}

pub mod cryptography {

    use orion::{aead, kex};

    use crate::plugins::Plugin;

    pub fn generate_keys(plugin: impl Plugin) {}

    // Takes in server public key and the received session ID, should attach the shared keys to the Plugin state.
    pub fn client_kex_handler(
        agency_key: Vec<u8>,
        session_id: Vec<u8>,
    ) -> super::core::Result<(kex::PublicKey, String)> {
        let public_key_slice: &[u8] = agency_key.as_slice();
        let server_public_key = kex::PublicKey::from_slice(public_key_slice)?;
        Ok((server_public_key, String::from_utf8(serialized_session_id)?))
    }

    // Takes in Client Key as arguments and returns server public key, and session ID for client
    pub fn server_kex_handler(
        agency_key: Vec<u8>,
    ) -> super::core::Result<(kex::SessionKeys, String)> {
        let session_server = kex::EphemeralClientSession::new()?;
        let client_public_key = session_server.public_key();

        //using the public value, generate the secrets.
        let session_keys = session_server.establish_with_server(&server_public_key)?;
        Ok((session_keys, session_id))
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
