use crate::protocol::{Addr, Message, MsgType, UpperProto};
use crate::transports::sample_looping_transport;
use crossbeam_queue::{ArrayQueue, PopError, PushError};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use uuid::Uuid;
use std::option::Option;
use std::sync::Arc;
use log::{info, debug};

#[derive(Clone)]
pub struct InDispatcher {
    passed_pkt: Vec<Uuid>,
    addr: Addr,
    counter: u64,
    registered_callbacks: HashMap<UpperProto, fn(&Message, Addr) -> Option<Message>>,
}

impl InDispatcher {
    pub fn new(addr: Addr) -> InDispatcher {
        InDispatcher {
            passed_pkt: vec![],
            addr,
            counter: 0,
            registered_callbacks: Default::default(),
        }
    }
    pub fn register_callback(&mut self, proto: UpperProto, func: fn(&Message, Addr) -> Option<Message>) {
        self.registered_callbacks.insert(proto, func);
    }
    pub fn dispatch(
        &mut self,
        input_queue: &ArrayQueue<Message>,
        relay_queue: &ArrayQueue<Message>,
    ) {
        loop {
            // println!("{:#?}", receiver);
            let mut msg = match input_queue.pop() {
                Ok(message) => message,
                _ => continue,
            };
            debug!("Message received.");
            let m_id = &msg.id;
            self.counter += 1;
            msg.ttl -= 1;
            if !self.passed_pkt.contains(m_id)
                & ((self.addr == msg.to) | (Addr(0, 0, 0, 0) == msg.to))
                & (msg.ttl > 0)
            {
                debug!("Message adressed to us.");
                if !self.registered_callbacks.contains_key(&msg.u_proto) {
                    self.passed_pkt.push(msg.id);
                    // TODO: Make an undefined handler
                    debug!("No handlers bind for {:?}. Skipping message for now.", &msg.u_proto);
                } else {
                    debug!("Moving message to {:?} handler.", &msg.u_proto);
                    self.passed_pkt.push(msg.id);
                    let func = self.registered_callbacks[&msg.u_proto];
                    match func(&msg, self.addr) {
                        Some(m) => {relay_queue.push(m).unwrap();}
                        _ => {}
                    };
                    continue;
                }
            } else {
                debug!("Message is not for us.");
                debug!("Moving it to relay queue.");
                relay_queue.push(msg).unwrap();
            }
        }
    }
}
