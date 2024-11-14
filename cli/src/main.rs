extern crate plugins;
extern crate utilities;

use utilities::plugins::Plugin;

fn main() {
    let plugin = plugins::MicrochipPlugin {
        public_key: Vec::new(),
        private_key: Vec::new()
    };

    let message = plugins::MicrochipMessage {
        shellcode: Vec::new(),
        source: String::from("Agent 1"),
        destination: String::from("Server 1"),
        agency_key: Vec::new(),
    };

    // Server logic here...

    plugins::MicrochipPlugin::agent_runtime(&plugin, message);
    
}
