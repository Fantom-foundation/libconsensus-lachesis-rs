extern crate serde;
extern crate serde_json;
use crate::event::event_hash::EventHash;
use failure::Error;
use std::sync::RwLock;
use std::fs::File;
use std::io::Read;

pub type PeerId = Vec<u8>;

#[derive(Serialize, Deserialize)]
pub struct LachesisPeerStruct {
    #[serde(rename="PubKeyHex")]
    id: PeerId,
    #[serde(rename="NetAddr")]
    net_addr: String,
    #[serde(skip)]
    used: i64,
    #[serde(skip)]
    height: i64,
    #[serde(skip)]
    in_degree: i64,
    #[serde(skip)]
    lock: RwLock<()>,
}

pub trait LachesisPeer {
    fn new(&self, id: PeerId, net_addr: String) -> LachesisPeerStruct;
    fn set_height(&mut self, height: i64);
    fn get_height(&self) -> i64;
    fn next_height(&mut self) -> i64;
    fn set_in_degree(&mut self, height: i64);
    fn get_in_degree(&self) -> i64;
    fn inc_in_degree(&mut self);
    fn get_peers(json_peer_path: String) -> Vec<LachesisPeerStruct>;
    fn inc_used(&mut self);
}

pub trait Peer<H>: Send + Sync {
    fn get_sync(&self, pk: PeerId, known: Option<&H>) -> Result<(EventHash, H), Error>;
    fn address(&self) -> String;
    fn id(&self) -> &PeerId;
}

impl LachesisPeer for LachesisPeerStruct {
    fn new (&self, id: PeerId, net_addr: String) -> LachesisPeerStruct {
            LachesisPeerStruct {
                id: id,
                net_addr: net_addr,
                used: 0,
                height: -1,
                in_degree: 0,
                lock: RwLock::new(()),
            }
    }
    fn set_height(&mut self, height: i64){
        let _ = self.lock.write().unwrap();
        self.height = height;
    }
    fn get_height(&self) -> i64 {
        let _ = self.lock.read().unwrap();
        return self.height;
    }
    fn next_height(&mut self) -> i64 {
        let _ = self.lock.write().unwrap();
        self.height += 1;
        return self.height;
    }
    fn set_in_degree(&mut self, in_degree: i64){
        let _ = self.lock.write().unwrap();
        self.in_degree = in_degree;
    }
    fn get_in_degree(&self) -> i64 {
        let _ = self.lock.read().unwrap();
        return self.in_degree;
    }
    fn inc_in_degree(&mut self) {
        let _ = self.lock.write().unwrap();
        self.in_degree += 1;
    }
    fn get_peers(json_peer_path: String) -> Vec<LachesisPeerStruct> {
        let mut file = File::open(json_peer_path).expect("get_peers(), open file");
        let mut data = String::new();
        file.read_to_string(&mut data).expect("reading from json file");
        let Ar: Vec<LachesisPeerStruct> = serde_json::from_str(&data).expect("desrialisation from json data");
        return Ar;
    }
    fn inc_used(&mut self) {
        let _ = self.lock.write().unwrap();
        self.used += 1;
    }
}
