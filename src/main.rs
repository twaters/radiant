extern crate radiant;
use radiant::arbiter_api;

fn main() {
    let ctx = zmq::Context::new();
	let arbiter = arbiter_api::ArbiterClient::new(&ctx, "CLIENT1", "tcp://127.0.0.1:5555");
	let arbiter = arbiter_api::ArbiterClient::new(&ctx, "CLIENT2", "tcp://127.0.0.1:5555");
	let arbiter = arbiter_api::ArbiterClient::new(&ctx, "CLIENT3", "tcp://127.0.0.1:5555");
	let arbiter = arbiter_api::ArbiterClient::new(&ctx, "CLIENT4", "tcp://127.0.0.1:5555");
	std::thread::sleep(std::time::Duration::from_millis(10000));
	
//	let req_socket = ctx.socket(zmq::ROUTER).unwrap();
//	let my_logical = "CLIENT";
//	let server_ident = "ARBITER";
//	
//	req_socket.set_identity(my_logical.as_bytes()).unwrap();
//	req_socket.connect("tcp://127.0.0.1:5555").unwrap();
//	
//	register(&server_ident, &req_socket);
//	
//	loop {
//		send_ping(&server_ident, &req_socket).unwrap();
//		if wait_for_message(3, 1000, &req_socket).is_ok() {
//			println!("got pong!");
//			std::thread::sleep(std::time::Duration::from_millis(1000));
//		} else {
//			println!("bad");
//		}
//	}
}
