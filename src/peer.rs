extern crate serde;
extern crate serde_json;
use crate::event::event_hash::EventHash;
use failure::Error;
use std::convert::From;
use std::fs::File;
use std::io::Read;
use std::sync::RwLock;

pub type PeerId = Vec<u8>;

// The very basic peer attributes used to specify a peer in the network
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct PeerBaseStruct {
    #[serde(rename = "PubKeyHex")]
    id: PeerId,
    #[serde(rename = "NetAddr")]
    net_addr: String,
}

// Lachesis peer attributes
#[derive(Serialize, Deserialize)]
pub struct LachesisPeerStruct {
    #[serde(rename = "PubKeyHex")]
    id: PeerId,
    #[serde(rename = "NetAddr")]
    net_addr: String,
    #[serde(skip, default)]
    used: u64,
    #[serde(skip, default)]
    height: u64,
    #[serde(skip, default)]
    in_degree: u64,
    #[serde(skip, default)]
    lock: RwLock<()>,
}

pub trait LachesisPeer {
    fn new(&self, id: PeerId, net_addr: String) -> LachesisPeerStruct;
    fn set_height(&mut self, height: u64);
    fn get_height(&self) -> u64;
    fn next_height(&mut self) -> u64;
    fn set_in_degree(&mut self, in_degree: u64);
    fn get_in_degree(&self) -> u64;
    fn inc_in_degree(&mut self);
    fn get_peers_from_file(json_peer_path: String) -> Result<Vec<LachesisPeerStruct>, Error>;
    fn inc_used(&mut self);
}

pub trait Peer<H>: Send + Sync {
    fn get_sync(&self, pk: PeerId, known: Option<&H>) -> Result<(EventHash, H), Error>;
    fn address(&self) -> String;
    fn id(&self) -> &PeerId;
}

impl From<&PeerBaseStruct> for LachesisPeerStruct {
    fn from(p: &PeerBaseStruct) -> Self {
        LachesisPeerStruct {
            id: p.id.clone(),
            net_addr: p.net_addr.clone(),
            used: 0,
            height: 0,
            in_degree: 0,
            lock: RwLock::new(()),
        }
    }
}

impl LachesisPeer for LachesisPeerStruct {
    fn new(&self, id: PeerId, net_addr: String) -> LachesisPeerStruct {
        LachesisPeerStruct {
            id,
            net_addr,
            used: 0,
            height: 0,
            in_degree: 0,
            lock: RwLock::new(()),
        }
    }
    fn set_height(&mut self, height: u64) {
        let _ = self.lock.write().unwrap();
        self.height = height;
    }
    fn get_height(&self) -> u64 {
        let _ = self.lock.read().unwrap();
        self.height
    }
    fn next_height(&mut self) -> u64 {
        let _ = self.lock.write().unwrap();
        self.height += 1;
        self.height
    }
    fn set_in_degree(&mut self, in_degree: u64) {
        let _ = self.lock.write().unwrap();
        self.in_degree = in_degree;
    }
    fn get_in_degree(&self) -> u64 {
        let _ = self.lock.read().unwrap();
        self.in_degree
    }
    fn inc_in_degree(&mut self) {
        let _ = self.lock.write().unwrap();
        self.in_degree += 1;
    }
    fn get_peers_from_file(json_peer_path: String) -> Result<Vec<LachesisPeerStruct>, Error> {
        let mut file = File::open(json_peer_path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let bp: Vec<PeerBaseStruct> = serde_json::from_str(&data)?;
        Ok(bp.iter().map(LachesisPeerStruct::from).collect())
    }
    fn inc_used(&mut self) {
        let _ = self.lock.write().unwrap();
        self.used += 1;
    }
}
