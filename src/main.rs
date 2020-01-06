extern crate radiant;
use radiant::arbiter_api;

fn main() {
    let ctx = zmq::Context::new();
	let mut client1 = arbiter_api::ArbiterClient::new(&ctx, "CLIENT1");
	client1.add_message(arbiter_api::MessageType::Command, "c1 command".to_string());
	client1.add_message(arbiter_api::MessageType::Data, "c1 notif".to_string());
	client1.connect(&ctx, "tcp://127.0.0.1:5555");
	
	let mut client2 = arbiter_api::ArbiterClient::new(&ctx, "CLIENT2");
	client2.add_message(arbiter_api::MessageType::Command, "c2 command".to_string());
	client2.add_message(arbiter_api::MessageType::Data, "c2 notif".to_string());
	client2.connect(&ctx, "tcp://127.0.0.1:5555");
	
	let mut client3 = arbiter_api::ArbiterClient::new(&ctx, "CLIENT3");
	client3.add_message(arbiter_api::MessageType::Command, "c3 command".to_string());
	client3.add_message(arbiter_api::MessageType::Data, "c3 notif".to_string());
	client3.connect(&ctx, "tcp://127.0.0.1:5555");
	
	let mut client4 = arbiter_api::ArbiterClient::new(&ctx, "CLIENT4");
	client4.add_message(arbiter_api::MessageType::Command, "c4 command".to_string());
	client4.add_message(arbiter_api::MessageType::Data, "c4 notif".to_string());
	client4.connect(&ctx, "tcp://127.0.0.1:5555");
	
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
