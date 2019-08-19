use crate::errors::{EventError, EventErrorType};
use crate::peer::PeerId;
use bincode::serialize;
use failure::Error;
use ring::digest::{digest, SHA256};
use serde::Serialize;
use std::collections::HashMap;

pub mod event_hash;
pub mod event_signature;
pub mod parents;

use self::event_hash::EventHash;
use self::event_signature::EventSignature;
use self::parents::Parents;
use crate::libconsensus::TransactionType;
use crate::peer::PeerBaseStruct;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InternalTransaction {
    transaction_type: TransactionType,
    peer: PeerBaseStruct,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Event<P: Parents + Clone + Serialize> {
    #[serde(skip)]
    can_see: HashMap<PeerId, EventHash>,
    #[serde(skip)]
    famous: Option<bool>,
    transactions: Vec<Vec<u8>>,
    internal_transactions: Vec<InternalTransaction>,
    parents: Option<P>,
    timestamp: Option<u64>,
    creator: PeerId,
    signature: Option<EventSignature>,
    #[serde(skip)]
    round: Option<usize>,
    #[serde(skip)]
    round_received: Option<usize>,
}

impl<P: Parents + Clone + Serialize> Event<P> {
    pub fn new(
        transactions: Vec<Vec<u8>>,
        internal_transactions: Vec<InternalTransaction>,
        parents: Option<P>,
        creator: PeerId,
    ) -> Event<P> {
        Event {
            can_see: HashMap::new(),
            creator,
            famous: None,
            transactions,
            internal_transactions,
            parents,
            round: None,
            round_received: None,
            signature: None,
            timestamp: None,
        }
    }

    #[inline]
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = Some(timestamp);
    }

    #[inline]
    pub fn timestamp(&self) -> Result<u64, Error> {
        Ok(self.timestamp.ok_or_else(|| match self.hash() {
            Err(e) => e,
            Ok(h) => Error::from(EventError::new(EventErrorType::NoTimestamp { hash: h })),
        })?)
    }

    #[inline]
    pub fn set_round_received(&mut self, round_received: usize) {
        self.round_received = Some(round_received);
    }

    #[inline]
    pub fn is_self_parent(&self, hash: &EventHash) -> Result<bool, Error> {
        let mut error: Option<Error> = None;
        let r = self
            .parents
            .clone()
            .map(|p| match p.self_parent() {
                Ok(self_parent) => self_parent == hash.clone(),
                Err(e) => {
                    error = Some(e);
                    false
                }
            })
            .unwrap_or(false);
        if let Some(e) = error {
            return Err(e);
        }
        Ok(r)
    }

    #[inline]
    pub fn signature(&self) -> Result<EventSignature, Error> {
        Ok(self.signature.clone().ok_or_else(|| match self.hash() {
            Err(e) => e,
            Ok(h) => Error::from(EventError::new(EventErrorType::NoSignature { hash: h })),
        })?)
    }

    #[inline]
    pub fn transactions(&self) -> Vec<Vec<u8>> {
        self.transactions.clone()
    }

    #[inline]
    pub fn famous(&mut self, famous: bool) {
        self.famous = Some(famous)
    }

    #[inline]
    pub fn is_famous(&self) -> bool {
        self.famous.unwrap_or(false)
    }

    #[inline]
    pub fn is_undefined(&self) -> bool {
        self.famous.is_none()
    }

    #[inline]
    pub fn can_see(&self) -> &HashMap<PeerId, EventHash> {
        &self.can_see
    }

    #[inline]
    pub fn set_can_see(&mut self, can_see: HashMap<PeerId, EventHash>) {
        self.can_see = can_see;
    }

    #[inline]
    pub fn round(&self) -> Result<usize, Error> {
        Ok(self.round.ok_or_else(|| match self.hash() {
            Err(e) => e,
            Ok(h) => Error::from(EventError::new(EventErrorType::RoundNotSet { hash: h })),
        })?)
    }

    #[inline]
    pub fn maybe_round(&self) -> Option<usize> {
        self.round
    }

    #[inline]
    pub fn add_can_see(&mut self, peer: PeerId, hash: EventHash) {
        self.can_see.insert(peer, hash);
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.parents.is_none()
    }

    #[inline]
    pub fn self_parent(&self) -> Result<EventHash, Error> {
        let mut error: Option<EventError> = None;
        let none_error = format_err!("self_parent() returned None");

        match self
            .parents
            .clone()
            .map(|p| match p.self_parent() {
                Ok(sp) => Some(sp),
                Err(e) => {
                    debug!(target: "event", "{}", e);

                    let hash: EventHash = match self.hash() {
                        Ok(hash) => hash,
                        Err(e) => {
                            debug!(target: "hash", "{}", e);
                            EventHash([0; 32])
                        }
                    };
                    error = Some(EventError::new(EventErrorType::NoSelfParent { hash }));
                    None
                }
            })
            .filter(Option::is_some)
            .unwrap()
        {
            Some(p) => Ok(p),
            None => Err(if let Some(e) = error {
                Error::from(e)
            } else {
                none_error
            }),
        }
    }

    #[inline]
    pub fn parents(&self) -> &Option<P> {
        &self.parents
    }

    #[inline]
    pub fn creator(&self) -> &PeerId {
        &self.creator
    }

    pub fn sign(&mut self, signature: EventSignature) {
        self.signature = Some(signature);
    }

    #[inline]
    pub fn set_round(&mut self, round: usize) {
        self.round = Some(round);
    }

    pub fn hash(&self) -> Result<EventHash, Error> {
        let value = (
            self.transactions.clone(),
            self.parents.clone(),
            self.timestamp,
            self.creator.clone(),
        );
        let bytes = serialize(&value)?;
        Ok(EventHash::new(digest(&SHA256, bytes.as_ref()).as_ref()))
    }

    pub fn is_valid(&self, hash: &EventHash) -> Result<bool, Error> {
        self.signature
            .clone()
            .map(|s| s.verify(&self, &self.creator))
            .unwrap_or_else(|| {
                Err(Error::from(EventError::new(
                    EventErrorType::UnsignedEvent { hash: self.hash()? },
                )))
            })?;
        Ok(hash.as_ref() == self.hash()?.as_ref())
    }
}

proptest! {
    #[test]
    fn root_event_shouldnt_have_self_parents(hash in ".*") {
        use crate::event::{EventHash, parents::ParentsPair};
        use ring::digest::{digest, SHA256};
        let event: Event<ParentsPair> = Event::new(vec![], vec![], None, vec![]);
        let hash = EventHash::new(digest(&SHA256, hash.as_bytes()).as_ref());
        assert!(!event.is_self_parent(&hash).unwrap())
    }

    #[test]
    fn it_should_report_correctly_self_parent(self_parent_hash in ".*", p_try in ".*") {
        use crate::event::{EventHash, parents::ParentsPair};
        use ring::digest::{digest, SHA256};
        let self_parent = EventHash::new(digest(&SHA256, self_parent_hash.as_bytes()).as_ref());
        let other_parent = EventHash::new(digest(&SHA256, b"fish").as_ref());
        let event = Event::new(vec![], vec![], Some(ParentsPair(self_parent.clone(), other_parent)), vec![]);
        let hash = EventHash::new(digest(&SHA256, p_try.as_bytes()).as_ref());
        assert!(event.is_self_parent(&self_parent).unwrap());
        assert_eq!(self_parent_hash == p_try, event.is_self_parent(&hash).unwrap())
    }

    #[test]
    fn it_should_have_different_hashes_on_different_transactions(tx1 in "[a-z]*", tx2 in "[a-z]*") {
        use crate::event::parents::ParentsPair;
        let event1: Event<ParentsPair> = Event::new(vec![tx1.as_bytes().to_vec()], vec![], None, vec![]);
        let event2: Event<ParentsPair> = Event::new(vec![tx2.as_bytes().to_vec()], vec![], None, vec![]);
        let event3: Event<ParentsPair> = Event::new(vec![tx2.as_bytes().to_vec()], vec![], None, vec![]);
        let hash1 = event1.hash().unwrap();
        let hash2 = event2.hash().unwrap();
        let hash3 = event3.hash().unwrap();
        assert!(hash2 == hash3);
        assert_eq!(tx1 == tx2, hash1 == hash2);
    }

    #[test]
    fn it_should_have_different_hashes_on_different_self_parents(tx1 in ".*", tx2 in ".*") {
        use crate::event::{EventHash, parents::ParentsPair};
        use ring::digest::{digest, SHA256};
        let other_parent = EventHash::new(digest(&SHA256, b"42").as_ref());
        let self_parent1 = EventHash::new(digest(&SHA256, tx1.as_bytes()).as_ref());
        let self_parent2 = EventHash::new(digest(&SHA256, tx2.as_bytes()).as_ref());
        let self_parent3 = EventHash::new(digest(&SHA256, tx2.as_bytes()).as_ref());
        let event1 = Event::new(vec![], vec![], Some(ParentsPair(self_parent1, other_parent.clone())), vec![]);
        let event2 = Event::new(vec![], vec![], Some(ParentsPair(self_parent2, other_parent.clone())), vec![]);
        let event3 = Event::new(vec![], vec![], Some(ParentsPair(self_parent3, other_parent.clone())), vec![]);
        let hash1 = event1.hash().unwrap();
        let hash2 = event2.hash().unwrap();
        let hash3 = event3.hash().unwrap();
        assert!(hash2 == hash3);
        assert_eq!(tx1 == tx2, hash1 == hash2);
    }

    #[test]
    fn it_should_have_different_hashes_on_different_other_parents(tx1 in ".*", tx2 in ".*") {
        use crate::event::{EventHash, parents::ParentsPair};
        use ring::digest::{digest, SHA256};
        let self_parent = EventHash::new(digest(&SHA256, b"42").as_ref());
        let other_parent1 = EventHash::new(digest(&SHA256, tx1.as_bytes()).as_ref());
        let other_parent2 = EventHash::new(digest(&SHA256, tx2.as_bytes()).as_ref());
        let other_parent3 = EventHash::new(digest(&SHA256, tx2.as_bytes()).as_ref());
        let event1 = Event::new(vec![], vec![], Some(ParentsPair(self_parent.clone(), other_parent1)), vec![]);
        let event2 = Event::new(vec![], vec![], Some(ParentsPair(self_parent.clone(), other_parent2)), vec![]);
        let event3 = Event::new(vec![], vec![], Some(ParentsPair(self_parent.clone(), other_parent3)), vec![]);
        let hash1 = event1.hash().unwrap();
        let hash2 = event2.hash().unwrap();
        let hash3 = event3.hash().unwrap();
        assert!(hash2 == hash3);
        assert_eq!(tx1 == tx2, hash1 == hash2);
    }

    #[test]
    fn it_should_have_different_hash_on_different_creators(c1 in ".*", c2 in ".*") {
        use crate::event::parents::ParentsPair;
        let event1: Event<ParentsPair> = Event::new(vec![], vec![], None, c1.as_bytes().to_vec());
        let event2: Event<ParentsPair> = Event::new(vec![], vec![], None, c2.as_bytes().to_vec());
        let event3: Event<ParentsPair> = Event::new(vec![], vec![], None, c2.as_bytes().to_vec());
        let hash1 = event1.hash().unwrap();
        let hash2 = event2.hash().unwrap();
        let hash3 = event3.hash().unwrap();
        assert!(hash2 == hash3);
        assert_eq!(c1 == c2, hash1 == hash2);
    }

    #[test]
    fn it_should_have_different_hash_on_different_timestamps(s1 in 0u64..10000, s2 in 0u64..10000) {
        use crate::event::parents::ParentsPair;
        let mut event1: Event<ParentsPair> = Event::new(vec![], vec![], None, vec![]);
        let mut event2: Event<ParentsPair> = Event::new(vec![], vec![], None, vec![]);
        let mut event3: Event<ParentsPair> = Event::new(vec![], vec![], None, vec![]);
        event1.set_timestamp(s1);
        event2.set_timestamp(s2);
        event3.set_timestamp(s2);
        let hash1 = event1.hash().unwrap();
        let hash2 = event2.hash().unwrap();
        let hash3 = event3.hash().unwrap();
        assert!(hash2 == hash3);
        assert_eq!(s1 == s2, hash1 == hash2);
    }
}

#[cfg(test)]
mod tests {
    use crate::event::{parents::ParentsPair, Event, EventHash, EventSignature};
    use ring::digest::{digest, SHA256};
    use ring::{rand, signature};

    #[test]
    fn it_should_succeed_when_verifying_correct_event() {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp =
            signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes)).unwrap();
        let mut event: Event<ParentsPair> =
            Event::new(vec![], vec![], None, kp.public_key_bytes().to_vec());
        let hash = event.hash().unwrap();
        let sign = kp.sign(hash.as_ref());
        let event_signature = EventSignature::new(sign.as_ref());
        event.sign(event_signature);
        assert!(event.is_valid(&hash).unwrap());
    }

    #[test]
    fn it_shouldnt_succeed_when_verifying_correct_event_with_wrong_hash() {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp =
            signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes)).unwrap();
        let mut event: Event<ParentsPair> =
            Event::new(vec![], vec![], None, kp.public_key_bytes().to_vec());
        let hash = event.hash().unwrap();
        let sign = kp.sign(hash.as_ref());
        let event_signature = EventSignature::new(sign.as_ref());
        let wrong_hash = EventHash::new(digest(&SHA256, b"42").as_ref());
        event.sign(event_signature);
        assert!(!event.is_valid(&wrong_hash).unwrap());
    }

    #[test]
    #[should_panic(expected = "Unspecified")]
    fn it_should_error_when_verifying_wrong_event() {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let kp =
            signature::Ed25519KeyPair::from_pkcs8(untrusted::Input::from(&pkcs8_bytes)).unwrap();
        let mut event: Event<ParentsPair> = Event::new(vec![], vec![], None, vec![]);
        let hash = event.hash().unwrap();
        let sign = kp.sign(hash.as_ref());
        let event_signature = EventSignature::new(sign.as_ref());
        event.sign(event_signature);
        assert!(!event.is_valid(&hash).unwrap());
    }
}
