//! The actor trait
use crate::error::Error;
use crate::message::{Request, Response};
use crate::runtime::Event;
use std::sync::mpsc;

/// The Actor trait that you need to implement
pub trait Actor {
    /// Initiate node with a name and a topology
    fn init(&mut self, node_id: &str, node_ids: Vec<String>) -> Result<(), Error>;
    /// Receive a request. Will answer with a Vec of Responses.
    fn receive(&mut self, request: &Request) -> Result<Vec<Response>, Error>;
    /// Send gossip to other node
    fn gossip(&mut self) -> Result<Vec<Response>, Error>;

    /// Inject Sender
    fn inject_sender(&mut self, sender: mpsc::Sender<Event>);
}
