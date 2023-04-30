use crate::helper::{compare_knowledge, convert_map};
use crate::protocol::Body::Gossip;
use crate::protocol::Message;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::process;

type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub struct Node {
    msg_counter: usize,
    guid: Option<String>,
    pub node_id: Option<String>,
    pub node_ids: Option<Vec<String>>,
    known: Vec<usize>,
    topology: BTreeMap<String, Vec<String>>,
    remote_known: BTreeMap<String, Vec<usize>>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            msg_counter: 0,
            node_id: None,
            node_ids: None,
            guid: None,
            known: vec![],
            topology: BTreeMap::new(),
            remote_known: BTreeMap::new(),
        }
    }

    pub fn next_mid(&mut self) -> usize {
        self.msg_counter += 1;
        self.msg_counter
    }

    pub fn get_guid(&mut self) -> Res<Option<String>> {
        match (self.guid.is_some(), self.node_id.is_some()) {
            (true, true) | (true, false) => Ok(self.guid.clone()),
            (false, true) => {
                self.guid = Some(format!(
                    "{}{}",
                    self.node_id.clone().unwrap(),
                    process::id()
                ));
                Ok(self.guid.clone())
            }
            (false, false) => Ok(None),
        }
    }

    pub fn save(&mut self, val: usize) -> Res<()> {
        self.known.push(val);
        self.known.dedup();
        self.known.sort();
        Ok(())
    }

    pub fn save_all(&mut self, vals: Vec<usize>) -> Res<()> {
        for v in vals{
            self.known.push(v);
        }
        self.known.dedup();
        self.known.sort();
        Ok(())
    }

    pub fn load(&self) -> Res<Vec<usize>> {
        Ok(self.known.clone())
    }

    pub fn update_topo(&mut self, topology: Map<String, Value>) -> Res<()> {
        for t in &topology {
            if !self.remote_known.contains_key(&t.0.to_string()) && self.node_id.clone().unwrap() != t.0.to_string(){
                self.remote_known.insert(t.0.to_string(), vec![]);
            }
        }
        self.topology = convert_map(topology);
        Ok(())
    }

    pub fn talk(&mut self) -> Option<Vec<Message>> {
        if self.topology.is_empty() && self.node_id.is_none() {
            return None;
        } 
        let mut messages: Vec<Message> = vec![];
        for (remote, known) in &self.remote_known.clone() {
            let to_send = compare_knowledge(self.known.clone(), known.to_vec());
            if !to_send.is_empty() {
                messages.push(Message {
                    src: self.node_id.clone().unwrap(),
                    dest: remote.to_string(),
                    body: Gossip {
                        msg_id: self.next_mid(),
                        news: to_send,
                    },
                })
            }
        }
        return Some(messages);
    }

    pub fn update_remote_known(&mut self, remote: String, knowledge: Vec<usize>) -> Res<()> {
        // Merge the existing vector of remote node with the new vector.
        if let Some(existing_values) = self.remote_known.get_mut(&remote) {
            // Remove any duplicates from the new vector.
            let mut unique_new_values = knowledge.clone();
            unique_new_values.retain(|value| !existing_values.contains(value));
            // Extend the existing vector with the new values.
            existing_values.extend(unique_new_values);
        } else {
            // If this case does happen it's likely to do with having known remote node without havinf a correct topology
            panic!("This should never happen");
            // If the key doesn't exist in the map, insert the new vector.
            // map.insert(remote, knowledge);
        }
        Ok(())
    }

    pub fn get_remote_known(&mut self, remote: String) -> Res<Vec<usize>> {
        let knownledge_vec = self.remote_known.get(&remote).unwrap().clone();
        Ok(knownledge_vec)
    }
}
