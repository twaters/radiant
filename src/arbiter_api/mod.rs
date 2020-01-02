use std::collections::HashMap;

#[derive(Copy, Clone)]
enum MessageType {
	Register = 0,
	AcceptConnection = 1,
	Deregister = 2,
	Ping = 3,
	Pong = 4,
	UNKNOWN = 5,
}

impl MessageType {	
	fn from_u8(value : u8) -> MessageType {
		match value {
			0 => MessageType::Register,
			1 => MessageType::AcceptConnection,
			2 => MessageType::Deregister,
			3 => MessageType::Ping,
			4 => MessageType::Pong,
			_ => MessageType::UNKNOWN,
		}
	}
}

struct ArbiterMessage {
	identity : String,
	message_type : MessageType,
	data_frames : Vec<zmq::Message>,
}

impl ArbiterMessage {
	fn new(identity : &str, message_type : MessageType) -> ArbiterMessage {
		ArbiterMessage {
			identity : identity.to_string(),
			message_type,
			// Allocate room for 4 data frames
			data_frames : vec!(
				zmq::Message::new(),
				zmq::Message::new(),
				zmq::Message::new(),
				zmq::Message::new()),
		}
	}

	fn send(mut self, socket : &zmq::Socket) {
		// Add the header containing the message type to the data to send
		self.data_frames.insert(0, zmq::Message::from(vec!(0 as u8, self.message_type as u8)));
		
		// If the socket requires an empty delimiter for the message enveloping
		// to function properly, add it (this is for DEALER and ROUTER sockets)
		let socket_type = socket.get_socket_type().unwrap();
		if socket_type == zmq::SocketType::DEALER || socket_type == zmq::SocketType::ROUTER {
			self.data_frames.insert(0, zmq::Message::new());
		}
		
		// If the socket requiers an identity frame to properly route data,
		// append it (this is for ROUTER sockets, only)
		if socket_type == zmq::SocketType::ROUTER {
			self.data_frames.insert(0, zmq::Message::from(&self.identity));
		}
		
		// Calcualte and append the checksum
		self.append_checksum();
		
		// Now the full frame is ready to send, send it out the socket
		socket.send_multipart(self.data_frames, 0).unwrap();
	}	
	
	fn append_checksum(&mut self) {
		// TODO Implement the checksum
		self.data_frames.push(zmq::Message::new());
	}
	
//	fn validate(self) -> zmq::Result<()> {
//		if self.frames.len() != 8 {
//			println!("bad frame len {}", self.frames.len());
//			return Err(zmq::Error::EMSGSIZE);
//		} else if self.frames[2].len() != 2 {
//			println!("bad header len");
//			return Err(zmq::Error::EPROTO);
//		} else if self.frames[2][1] >= MessageType::UNKNOWN as u8{
//			println!("wrong message type");
//			return Err(zmq::Error::ENOTSUP);
//		} 
////		else if !is_checksum_valid(self) {
////			println!("checksum fail");
////			return Err(zmq::Error::EINVAL);
////		}
//	
//		Ok(())
//	}
	
	fn is_checksum_valid(&self) -> bool {
		true
	}
	
	fn get_message_type(&self) -> MessageType {
		self.message_type
	}
}

pub struct ArbiterClient {			
	server_identity : String,
	my_identity : String,
	network_state_socket : zmq::Socket,
}

impl ArbiterClient {
	pub fn new(ctx : &zmq::Context, identity : &str, connection_string : &str) -> ArbiterClient {
		// Construct an ArbiterClient struct
		let arbiter = ArbiterClient {
			server_identity : String::from("ARBITER"),
			my_identity : String::from(identity),
			network_state_socket : ctx.socket(zmq::SUB).unwrap(),
		};
		
		// Setup a socket for processing registration and pings
		let registration_socket = ctx.socket(zmq::DEALER).unwrap();
		let server_identity2 = arbiter.server_identity.clone();
		registration_socket.set_identity(identity.as_bytes()).unwrap();
		registration_socket.connect(connection_string).unwrap();
		
		// Start the thread to process registration and pings
		std::thread::Builder::new().name("Arbitration".to_string())
			.spawn(move || {
				// First, register with the arbiter
				ArbiterClient::register(&registration_socket, &server_identity2);
	
				// Now, just ping forever
				ArbiterClient::ping (&registration_socket, &server_identity2);
			}).unwrap();
		
		arbiter
	}
	
	
	
	fn register(registration_socket : &zmq::Socket, server_identity : &str) {
		let mut cont = true;
		while cont {
			let register_msg = ArbiterMessage::new("", MessageType::Register);
			register_msg.send(registration_socket);
			cont = wait_for_message(MessageType::AcceptConnection, 2000, &registration_socket).is_err();
			println!("regi cont: {}", cont);
		}

		println!("Registered");
	}

	fn ping(socket : &zmq::Socket, server_identity : &str) {
		loop {
			let ping = ArbiterMessage::new("", MessageType::Ping);
			ping.send(socket);
			if wait_for_message(MessageType::Pong, 1000, &socket).is_ok() {
				println!("got pong");
				std::thread::sleep(std::time::Duration::from_millis(1000));
			} else {
				println!("NO PING REPLY...DO SOMETHING"); // TODO
			}
		}
	}
}

pub struct ArbiterServer {
	my_identity : String,
}

impl ArbiterServer {
	pub fn new(ctx : &zmq::Context, server_identity : &str) -> ArbiterServer {
		ArbiterServer {
			my_identity : server_identity.to_string(),
		}
	}
	
	pub fn run_server (mut self, ctx : &zmq::Context, pub_state_bind : &str, node_request_bind : &str) {
		// Startup the state publisher thread
		let pub_state_socket = ctx.socket(zmq::PUB).unwrap();
		pub_state_socket.bind(pub_state_bind).unwrap();
		std::thread::Builder::new().name("State_Publisher".to_string())
			.spawn(move || {
				ArbiterServer::state_publisher(&pub_state_socket);
			}).unwrap();
			
		// Setup the main router socket for processing node requests
		let node_request_socket = ctx.socket(zmq::ROUTER).unwrap();
		node_request_socket.set_identity(self.my_identity.as_bytes()).unwrap();
		node_request_socket.bind(node_request_bind).unwrap();
		ArbiterServer::process_node_requests(node_request_socket, &ctx, self.my_identity);
	}
	
	fn state_publisher(socket : &zmq::Socket) {
		let mut i = 0;
		loop {
			i += 1;
			let state = zmq::Message::from_slice(i.to_string().as_bytes());
			socket.send(state, 0).unwrap();

			println!("published {}", i);
			std::thread::sleep(std::time::Duration::from_millis(1000));
		}
	}
	
	fn process_node_requests(node_request_socket : zmq::Socket, ctx : &zmq::Context, server_identity : String) {
		let mut nodes = HashMap::new();
		nodes.insert(server_identity.clone(), node_request_socket);
	
		loop {
			// Build a list of poll items to use
			let mut poll_items = Vec::new();
			for (_, socket) in &nodes {
				poll_items.push(socket.as_poll_item(zmq::PollEvents::POLLIN));
			}
			
			// Wait for a message from any internal thread or the external socket
			zmq::poll(&mut poll_items[..], -1);
			
			// Monitor each socket for any data and build a list of messages
			// that need processed
			let mut messages = Vec::new();
			for (identity, socket) in &nodes {
				let events = socket.get_events().unwrap();
				if events.contains(zmq::POLLIN) {
					// TODO this receives only on the main node rquest socket
					// But we need to add polling for the spawned child threads, as well
					messages.push(receive_multi(&socket));
				}
			}

			ArbiterServer::process_messages(messages, &mut nodes, &ctx, &server_identity);
		}
	}	
	
	fn process_messages(messages : Vec<ArbiterMessage>, nodes : &mut HashMap<String, zmq::Socket>, ctx : &zmq::Context, server_identity : &String) {
		for message in messages {
			match message.get_message_type() {
				MessageType::Register => {
					println!("got register");
					// Register a new node and spawn the child thread
					if let Some(new_socket) = ArbiterServer::register_node(&message, &nodes, &ctx) {
						nodes.insert(message.identity.clone(), new_socket);
					}
			
					// Send a connection accept (TODO: Handle a return value on register_node before sending)
					let accept_msg = ArbiterMessage::new(&message.identity, MessageType::AcceptConnection);
					if let Some(socket) = nodes.get(server_identity) {
						accept_msg.send(&socket);
					}
				},
				MessageType::Deregister => {
					// Remove the client from the list of nodes
					println!("removing {}", message.data_frames[0].as_str().unwrap());
					nodes.remove(message.data_frames[0].as_str().unwrap());
				},
				MessageType::Ping => {
					if let Some(node) = nodes.get(&message.identity) {
						// TODO SEems unfortunate that we need to re-build the entire ping message
						// just to forward it to the child thread
						let child_ping = ArbiterMessage::new("", MessageType::Ping);
						child_ping.send(&node);
						
						let pong_msg = ArbiterMessage::new(&message.identity, MessageType::Pong);
						if let Some(socket) = nodes.get(server_identity) {
							pong_msg.send(&socket);
						}
					} else {
						println!("Ping received for invalid identity");
					}
				},
				_ => println!("Invalid message")
			}
		}
	}
	
	fn register_node(registration_message : &ArbiterMessage, children : &HashMap<String, zmq::Socket>, ctx : &zmq::Context) -> Option<zmq::Socket> {
		// Safely check the application (identity) name
		if !registration_message.identity.is_empty() {
			// Check whether the child already exists
			if let Some(_child) = children.get(&registration_message.identity) {
				println!("Already here");
			} else {
				// Create a new socket and tie it with the identity...this is how we will communicate with the child processing thread
				let mut child_binding = "inproc://".to_string();
				child_binding.push_str(&registration_message.identity);
	
				let parent_socket = ctx.socket(zmq::PAIR).unwrap();
				parent_socket.bind(&child_binding).unwrap();

				ArbiterServer::spawn_child(&registration_message, &child_binding, &ctx);
				return Some(parent_socket);
			}
		}
		else {
			println!("Invalid identity name");
		}
		
		None
	}
	
	fn spawn_child(registration_message : &ArbiterMessage, child_binding : &String, ctx : &zmq::Context) {
		let child_socket = ctx.socket(zmq::PAIR).unwrap();
		child_socket.connect(&child_binding).unwrap();
	
		let ident = registration_message.identity.to_string();

		// Start the child processing thread to handle pings and de-registrations
		std::thread::Builder::new().name(ident.to_string())
			.spawn(move || {
				loop {
					if wait_for_message(MessageType::Ping, 5000, &child_socket).is_err() {
						// Send deregistration to the parent thread
						let mut dereg_msg = ArbiterMessage::new(&ident, MessageType::Deregister);
						dereg_msg.data_frames[0] = zmq::Message::from(&ident);
						dereg_msg.send(&child_socket);
						println!("should deregister..");
						// Exit the thread
						break;
					}
				}
			}).unwrap();
	}
}

fn receive_multi(socket : &zmq::Socket) -> ArbiterMessage {
	let mut message = Vec::new();
	let mut more = true;

	// Receive everything available in the mssage
	while more {
		let mut part_msg = zmq::Message::new();
		socket.recv(&mut part_msg, 0).unwrap();
		more = part_msg.get_more();
		message.push(part_msg);
	}

	// Now find where the header is in the message, which
	// depends on the socket enveloping, so for router and
	// dealer sockets, jump past the enveloping into our data
	// the first frame in our data is our message header
	let mut header_index = 0;
	let mut identity = String::new();
	let socket_type = socket.get_socket_type().unwrap();
	if socket_type == zmq::SocketType::ROUTER {
		header_index = 2;
		
		// Also, save the identity
		if !message.is_empty() {
			identity = message[0].as_str().unwrap().to_string();
		}
	} else if socket_type == zmq::SocketType::DEALER {
		header_index = 1;
	}
	
	// Do some minor validation to make sure that the received
	// message is large enough to construct an Arbiter Message, 
	// that is, it has a header frame and we can determine the
	// message type
	if message.len() == header_index + 6 && message[header_index].len() == 2 {
		ArbiterMessage {
			identity,
			message_type : MessageType::from_u8(message[header_index][1]),
			data_frames : message.split_off(header_index + 1),
		}
	} else {
		// The received message was not big enough to be valid.
		// For now, return an empty message, but eventually we should
		// instead return an error (option?) struct TODO
		ArbiterMessage {
			identity,
			message_type : MessageType::UNKNOWN,
			data_frames : Vec::new(),
		}
	}
}

fn receive_message(socket : &zmq::Socket) -> zmq::Result<ArbiterMessage> {
	// Get the full message from the socket
	let mut message = receive_multi(&socket);
	
	// Remove the empty delimiter frame
	message.data_frames.remove(0);
	
	Ok(message)
//	let valid = message.validate();
//	if valid.is_ok() {
//		Ok(message)
//	} else {
//		valid
//	}
}

fn wait_for_message(message_type : MessageType, timeout_ms : i64, socket : &zmq::Socket) ->zmq::Result<ArbiterMessage> {
	// Poll on the socket for any received data, once a message is received, pull it off the socket
	if socket.poll(zmq::PollEvents::POLLIN, timeout_ms).unwrap() > 0 {
		let result = receive_message(&socket);
		if let Ok(message) = result {
			if message.get_message_type() as u8 == message_type as u8 {
				println!("got it");
				Ok(message)
			} else {
				println!("wrong type");
				// If we receive a message that is not what we expected,
				// we will drop the message.  TODO, should we keep it somehow?
				Err(zmq::Error::EPROTO)
			}
		} else {
			println!("receive failed early");
			result
		}
	} else {
		println!("wait proto");
		Err(zmq::Error::EPROTO)
	}
}