extern crate radiant;
use radiant::arbiter_api;

fn main() {
	// Get the command-line arguments
	let mut node_request_bind = "tcp://*:5555";
	let mut arbiter_ident = "ARBITER";
	let mut pub_state_bind = "tcp://*:5556";
	
	let args: Vec<String> = std::env::args().collect();
	
	if args.len() > 1 {
		node_request_bind = &args[1];
	}
	if args.len() > 2 {
		arbiter_ident = &args[2];
	}
	if args.len() > 3 {
		pub_state_bind = &args[3];
	}
	
	// Setup the ZeroMQ context for the process
    let ctx = zmq::Context::new();

	// Start the server...
	let arbiter = arbiter_api::ArbiterServer::new(&ctx, &arbiter_ident);
	arbiter.run_server(&ctx, &pub_state_bind, &node_request_bind);
}

