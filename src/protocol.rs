use crate::helper::*;
use crate::node::Node;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Body {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk {
        msg_id: usize,
        in_reply_to: usize,
    },
    Echo {
        msg_id: usize,
        echo: String,
    },
    EchoOk {
        msg_id: usize,
        echo: String,
        in_reply_to: usize,
    },
    Generate {
        msg_id: usize,
    },
    GenerateOk {
        msg_id: usize,
        id: String,
        in_reply_to: usize,
    },
    Broadcast {
        msg_id: usize,
        message: usize,
    },
    BroadcastOk {
        msg_id: usize,
        in_reply_to: usize,
    },
    Read {
        msg_id: usize,
    },
    ReadOk {
        msg_id: usize,
        in_reply_to: usize,
        messages: Vec<usize>,
    },
    Topology {
        msg_id: usize,
        topology: Map<String, Value>,
    },
    TopologyOk {
        msg_id: usize,
        in_reply_to: usize,
    },
    Gossip {
        msg_id: usize,
        news: Vec<usize>,
    },
    GossipOk {
        msg_id: usize,
        news: Vec<usize>,
        in_reply_to: usize,
    },
}
use Body::*;

impl Message {
    pub fn handle(&self, node: &mut Node) -> Res<Option<Message>> {
        let msg: Option<Message> = match &self.body {
            Init {
                msg_id,
                node_id,
                node_ids,
            } => Some(self.init_handle(node, node_id.to_string(), node_ids.to_vec(), *msg_id)?),
            InitOk { .. } => None,
            Echo { msg_id, echo } => {
                Some(self.echo_handle(node.next_mid(), echo.to_string(), *msg_id)?)
            }
            EchoOk { .. } => None,
            Generate { msg_id, .. } => Some(self.generate_handle(node, *msg_id)?),
            GenerateOk { .. } => None,
            Broadcast {
                msg_id, message, ..
            } => Some(self.broadcast_handle(node, *msg_id, *message)?),
            BroadcastOk { .. } => None,
            Read { msg_id, .. } => Some(self.read_handle(node, *msg_id)?),
            ReadOk { .. } => None,
            Topology { msg_id, topology } => {
                Some(self.topology_handle(node, *msg_id, topology.clone())?)
            }
            TopologyOk { .. } => None,
            Gossip { msg_id, news } => Some(self.gossip_handle(node, *msg_id, news.clone())?),
            GossipOk { msg_id, news, .. } => self.gossip_ok_handle(node, *msg_id, news.clone())?,
        };
        Ok(msg)
    }

    fn reply(&self, body: Body) -> Res<Message> {
        Ok(Message {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: body,
        })
    }

    fn init_handle(
        &self,
        node: &mut Node,
        node_id: String,
        node_ids: Vec<String>,
        in_reply_to: usize,
    ) -> Res<Message> {
        node.node_id = Some(node_id);
        node.node_ids = Some(node_ids);
        self.reply(InitOk {
            msg_id: node.next_mid(),
            in_reply_to,
        })
    }

    fn echo_handle(&self, msg_id: usize, echo: String, in_reply_to: usize) -> Res<Message> {
        self.reply(EchoOk {
            msg_id,
            in_reply_to,
            echo,
        })
    }

    fn generate_handle(&self, node: &mut Node, in_reply_to: usize) -> Res<Message> {
        self.reply(GenerateOk {
            msg_id: node.next_mid(),
            in_reply_to,
            id: node.get_guid()?.unwrap(),
        })
    }

    fn broadcast_handle(
        &self,
        node: &mut Node,
        in_reply_to: usize,
        message: usize,
    ) -> Res<Message> {
        node.save(message)?;
        self.reply(BroadcastOk {
            msg_id: node.next_mid(),
            in_reply_to,
        })
    }

    fn read_handle(&self, node: &mut Node, in_reply_to: usize) -> Res<Message> {
        self.reply(ReadOk {
            msg_id: node.next_mid(),
            in_reply_to,
            messages: node.load()?,
        })
    }

    fn topology_handle(
        &self,
        node: &mut Node,
        in_reply_to: usize,
        topology: Map<String, Value>,
    ) -> Res<Message> {
        node.update_topo(topology)?;
        self.reply(TopologyOk {
            msg_id: node.next_mid(),
            in_reply_to,
        })
    }

    fn gossip_handle(
        &self,
        node: &mut Node,
        in_reply_to: usize,
        news_recv: Vec<usize>,
    ) -> Res<Message> {
        let local_unknown = compare_knowledge(news_recv.clone(), node.load()?);
        // let remote_unknown = compare_knowledge(node.load()?, news_recv.clone());
        node.save_all(local_unknown.clone())?;

        // Update known values of remote node
        node.update_remote_known(self.src.clone(), news_recv)?;

        let remote_known = node.get_remote_known(self.src.clone())?;
        let local_known = node.load()?;
        let to_send = compare_knowledge(remote_known.clone(), local_known.clone());

        self.reply(GossipOk {
            msg_id: node.next_mid(),
            in_reply_to,
            news: to_send,
        })
    }

    fn gossip_ok_handle(
        &self,
        node: &mut Node,
        in_reply_to: usize,
        news_recv: Vec<usize>,
    ) -> Res<Option<Message>> {
        if news_recv.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.gossip_handle(node, in_reply_to, news_recv)?))
        }
    }
}
