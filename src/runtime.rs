use crate::message::{Request, Response};
use crate::node::Node;
use log::error;
use serde::de::Error;
use serde_json::{Map, Value};
use std::io::stdin;

pub struct Runtime {
    node: Box<dyn Node>,
}

impl Runtime {
    pub fn new(node: Box<dyn Node>) -> Runtime {
        Runtime { node }
    }

    pub fn start(&mut self) {
        let mut buffer = String::new();
        loop {
            stdin()
                .read_line(&mut buffer)
                .expect("could not read stdin");

            let mut valid_json: Map<String, Value> = match serde_json::from_slice(buffer.as_bytes())
            {
                Ok(v) => v,
                Err(e) => {
                    error!("could not deserialize stdin as json: {}", e);
                    error!("stdin's content is {}", buffer);
                    continue;
                }
            };

            let message = match Request::try_from_json(&mut valid_json) {
                Ok(m) => m,
                Err(error) => {
                    error!("could not deserialize stdin as a Maelstrom json: {}", error);
                    continue;
                }
            };
            eprintln!("received {:?}", &message);

            if message.message_type.as_str().eq("init") {
                match self.handle_init(&message) {
                    Ok(_) => {}
                    Err(error) => {
                        error!(
                            "could not deserialize stdin as a Maelstrom init json: {}",
                            error
                        );
                        continue;
                    }
                }
            } else {
                match self.node.receive(&message) {
                    Ok(responses) => {
                        responses
                            .iter()
                            .for_each(|resp| self.create_response(&message, resp));
                    }
                    Err(_) => unimplemented!(),
                }
            }

            buffer.clear();
        }
    }
    fn handle_init(&mut self, message: &Request) -> Result<(), serde_json::Error> {
        let node_id = match message.body.get("node_id") {
            Some(Value::String(s)) => s,
            _ => {
                return Err(serde_json::Error::custom(
                    "could not find node_id as a string in init message",
                ))
            }
        };
        let node_ids: Vec<String> = match message.body.get("node_ids") {
            Some(Value::Array(ids)) => ids
                .iter()
                .map(|id| id.as_str())
                .filter(|maybe_str| maybe_str.is_some())
                .map(|id| String::from(id.unwrap()))
                .collect(),
            _ => {
                return Err(serde_json::Error::custom(
                    "could not find node_ids as a vec of string in init message",
                ))
            }
        };

        match self.node.init(node_id.as_str(), node_ids) {
            Ok(_) => self.create_init_response(message),
            Err(error) => unimplemented!("handle error on init {}", error),
        };
        Ok(())
    }

    fn create_response(&self, request: &Request, response: &Response) {
        let mut body = Map::new();
        body.insert(
            String::from("type"),
            Value::from(response.message_type.to_owned()),
        );
        if let Some(message_id) = response.message_id {
            body.insert(String::from("msg_id"), Value::from(message_id));
        }
        if let Some(in_reply_to) = response.in_reply_to {
            body.insert(String::from("in_reply_to"), Value::from(in_reply_to));
        }

        for (k, v) in response.body.iter() {
            body.insert(k.to_owned(), v.to_owned());
        }
        self.send_response(request, body);
    }

    fn create_init_response(&self, request: &Request) {
        let response = Response {
            message_type: "init_ok".to_string(),
            message_id: request.message_id.map(|n| n + 1),
            in_reply_to: request.message_id,
            body: Default::default(),
        };
        self.create_response(request, &response);
    }

    fn send_response(&self, request: &Request, body: Map<String, Value>) {
        let mut reply = Map::new();
        reply.insert(
            String::from("src"),
            Value::from(request.destination.clone()),
        );
        reply.insert(String::from("dest"), Value::from(request.source.clone()));
        reply.insert(String::from("body"), Value::from(body));
        eprintln!("reply: {:?}", &reply);

        let response = serde_json::to_string(&reply).expect("could not serialize");
        println!("{}", response)
    }
}