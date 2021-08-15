use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen};
use near_sdk::serde::{Deserialize, Serialize};
//use std::collections::HashMap;
use near_sdk::collections::UnorderedMap;
use base58::*;

#[warn(unused_imports)]
const DEFAULT_CONTEXT1: &'static str = "https://www.w3.org/ns/did/v1";
const DEFAULT_CONTEXT2: &'static str = "https://www.near.org/did/v1";

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct DID {
    /// Status is used to store the state of DID. There are two states of DID, valid and invalid.
    pub status: UnorderedMap<String, Status>,
    /// context.
    pub contexts: UnorderedMap<String, Vec<String>>,
    /// It is used to store all public key information corresponding to this DID
    pub public_key: UnorderedMap<String, PublicKeyList>,
    pub authentication: UnorderedMap<String, Vec<u32>>, //
    /// It is used to store all controller information corresponding to this DID. The controller has the authority to update the information of this did.
    pub controller: UnorderedMap<String, Vec<String>>,
    pub service: UnorderedMap<String, Vec<Service>>,
    /// Used to store the creation time of DID
    pub created: UnorderedMap<String, u64>,
    /// Used to store the update time of DID
    pub updated: UnorderedMap<String, u64>,
}

impl Default for DID {
    fn default() -> Self {
        Self {
            status: UnorderedMap::new(b"r".to_vec()),
            contexts:UnorderedMap::new(b"c".to_vec()),
            public_key: UnorderedMap::new(b"k".to_vec()),
            authentication: UnorderedMap::new(b"w".to_vec()),
            controller: UnorderedMap::new(b"a".to_vec()),
            service: UnorderedMap::new(b"d".to_vec()),
            created: UnorderedMap::new(b"j".to_vec()),
            updated: UnorderedMap::new(b"h".to_vec())
        }
    }
}

#[near_bindgen]
impl DID {
    /// register DID
    ///
    /// this method will store DID information on the chain
    ///
    /// this method will output log information in the following format,
    ///
    /// log information: "reg_did_using_account: did:near:abcde.testnet"
    pub fn reg_did_using_account(&mut self) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        let status = self.status.get(&did);
        assert!(status.is_none());

        self.status.insert(&did, &Status::VALID);
        self.public_key
            .insert(&did, &PublicKeyList::new_default(&did, account_pk));
        let index: u32 = 0;
        self.authentication.insert(&did, &vec![index]);
        self.created.insert(&did, &env::block_timestamp());

        let log_message = format!("reg_did_using_account: {}", &did);
        env::log(log_message.as_bytes());
    }

    /// deactivate_did DID
    ///
    /// this method will update DID to deactive status, this means the DID is a invalid.
    ///
    /// log information: "deactivate_did: did:near:abcde.testnet"
    pub fn deactivate_did(&mut self) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        let status = self.status.get(&did);
        assert!(status.is_some());
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        self.status.insert(&did, &Status::DEACTIVATED);
        self.contexts.remove(&did);
        self.public_key.remove(&did);
        self.authentication.remove(&did);
        self.controller.remove(&did);
        self.service.remove(&did);
        self.created.remove(&did);
        self.updated.remove(&did);

        let log_message = format!("deactivate_did: {}", &did);
        env::log(log_message.as_bytes());
    }

    /// add_controller
    ///
    /// this method will add a controller for the DID. controller is also a DID, it has the right to update the information of DID.
    ///
    /// parameter `controller` is also a DID
    ///
    /// log information: "add_controller, did: did:near:abcde.testnet, controller: did:near:abcdefg.testnet"
    ///
    pub fn add_controller(&mut self, controller: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        let log_message = format!("add_controller, did: {}, controller: {}", &did, &controller);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);
        check_did(&controller);
        let mut controller_list = self.controller.get(&did).unwrap_or(vec![]);
        if controller_list.contains(&controller) {
            env::panic(b"add_controller, controller exists")
        };

        controller_list.push(controller);
        self.controller.insert(&did, &controller_list);
        self.updated.insert(&did, &env::block_timestamp());
        env::log(log_message.as_bytes());
    }

    /// remove_controller
    ///
    /// this method will remove a controller for the DID. only the DID owner has the right to remove controller.
    ///
    /// parameter `controller` is also a DID
    ///
    /// log information: "remove_controller, did: did:near:abcde.testnet, controller: did:near:abcdefg.testnet"
    ///
    pub fn remove_controller(&mut self, controller: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let mut controller_list = self.controller.get(&did).unwrap();
        let index = controller_list
            .iter()
            .position(|x| x == &controller)
            .unwrap();
        controller_list.remove(index);
        self.controller.insert(&did, &controller_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!(
            "remove_controller, did:{}, controller: {}",
            &did, controller
        );
        env::log(log_message.as_bytes());
    }

    /// add_key
    ///
    /// this method will add a public key for the DID. only the DID owner has the right to add public key.
    ///
    /// parameter `controller` is also a DID
    ///
    /// log information: "add_key, did: did:near:abcde.testnet, public key: , controller: did:near:abcdefg.testnet"
    ///
    pub fn add_key(&mut self, pk: Vec<u8>, controller: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        let log_message = format!(
            "add_key, did:{}, public key: {:?}, controller: {}",
            &did, &pk, &controller
        );

        self.check_did_status(&did);
        check_did(&controller);
        let mut public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);
        if public_key_list.pk_exist(&pk) {
            env::panic(b"add_key, pk exists")
        }

        public_key_list.push(PublicKey::new_pk(&controller, pk));
        self.public_key.insert(&did, &public_key_list);
        self.updated.insert(&did, &env::block_timestamp());

        env::log(log_message.as_bytes());
    }

    /// deactivate_key
    ///
    /// this method will update a public key to deactive status. only the DID owner has the right to invoke this method.
    ///
    /// parameter `pk` is a public key
    ///
    /// log information: "deactivate_key, did: did:near:abcde.testnet, public key: "
    ///
    pub fn deactivate_key(&mut self, pk: Vec<u8>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let mut public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        public_key_list.deactivate_pk(&pk);
        self.public_key.insert(&did, &public_key_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!("deactivate_key, did:{}, public key: {:?}", &did, pk);
        env::log(log_message.as_bytes());
    }

    /// add_new_auth_key
    ///
    /// this method will add a public key to deactive status. only the DID owner has the right to invoke this method.
    ///
    /// `pk` is a public key
    ///
    /// `controller` is a DID
    ///
    /// log information: "add_new_auth_key, did: did:near:abcde.testnet, public key: ,controller: did:near:abcdefg.testnet"
    ///
    pub fn add_new_auth_key(&mut self, pk: Vec<u8>, controller: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        check_did(&controller);
        let mut public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);
        if public_key_list.pk_exist(&pk) {
            env::panic(b"add_new_auth_key, pk exists")
        }

        let log_message = format!(
            "add_new_auth_key, did:{}, public key: {:?}, controller: {}",
            &did, &pk, &controller
        );

        public_key_list.push(PublicKey::new_auth(&controller, pk));
        self.public_key.insert(&did, &public_key_list);
        let mut authentication_list = self.authentication.get(&did).unwrap();
        let index: u32 = (public_key_list.len() - 1) as u32;
        authentication_list.push(index);
        self.authentication.insert(&did, &authentication_list);
        self.updated.insert(&did, &env::block_timestamp());

        env::log(log_message.as_bytes());
    }

    /// set_auth_key
    ///
    /// this method will set the pk to authentication status. only the DID owner has the right to invoke this method.
    ///
    /// `pk` is a public key
    ///
    /// log information: "set_auth_key, did: did:near:abcde.testnet, public key: "
    ///
    pub fn set_auth_key(&mut self, pk: Vec<u8>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let mut public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let index = public_key_list.set_pk_auth(&pk);
        self.public_key.insert(&did, &public_key_list);
        let mut authentication_list = self.authentication.get(&did).unwrap();
        authentication_list.push(index as u32);
        self.authentication.insert(&did, &authentication_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!("set_auth_key, did:{}, public key: {:?}", &did, pk);
        env::log(log_message.as_bytes());
    }

    /// deactivate_auth_key
    ///
    /// this method will update the pk to authentication invalid status. only the DID owner has the right to invoke this method.
    ///
    /// `pk` is a public key
    ///
    /// log information: "deactivate_auth_key, did: did:near:abcde.testnet, public key: "
    ///
    pub fn deactivate_auth_key(&mut self, pk: Vec<u8>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let mut public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let index = public_key_list.remove_pk_auth(&pk);
        self.public_key.insert(&did, &public_key_list);
        let mut authentication_list = self.authentication.get(&did).unwrap();
        let i = authentication_list
            .iter()
            .position(|x| x == &(index as u32))
            .unwrap();
        authentication_list.remove(i);
        self.authentication.insert(&did, &authentication_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!("deactivate_auth_key, did:{}, public key: {:?}", &did, pk);
        env::log(log_message.as_bytes());
    }

    /// add_new_auth_key_by_controller
    ///
    /// this method will add new auth key by controller. need the controller's signature.
    ///
    /// `pk` is controller's public key
    ///
    /// log information: "add_new_auth_key_by_controller, did: did:near:abcde.testnet, public key: ,controller: did:near:abcdefg.testnet"
    ///
    pub fn add_new_auth_key_by_controller(&mut self, did: String, pk: Vec<u8>, controller: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let controller_did = gen_did(&account_id);

        self.check_did_status(&did);
        self.check_did_status(&controller_did);
        check_did(&controller);
        let controller_list = self.controller.get(&did).unwrap();
        if !controller_list.contains(&controller_did) {
            env::panic(b"add_new_auth_key_by_controller, signer is not controller")
        }
        let controller_public_key_list = self.public_key.get(&controller_did).unwrap();
        controller_public_key_list.check_pk_access(&account_pk);

        let mut public_key_list = self.public_key.get(&did).unwrap();
        if public_key_list.pk_exist(&pk) {
            env::panic(b"add_new_auth_key_by_controller, pk exists")
        }

        public_key_list.push(PublicKey::new_auth(&controller, pk.clone()));
        self.public_key.insert(&did, &public_key_list);
        let mut authentication_list = self.authentication.get(&did).unwrap();
        let index: u32 = public_key_list.len() - 1;
        authentication_list.push(index);
        self.authentication.insert(&did, &authentication_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!(
            "add_new_auth_key_by_controller, did:{}, public key: {:?}, controller: {}",
            &did, pk, controller
        );
        env::log(log_message.as_bytes());
    }

    /// set_auth_key_by_controller
    ///
    /// this method will set the public key to auth key by controller. need the controller's signature.
    ///
    /// `pk` is controller's public key
    ///
    /// log information: "set_auth_key_by_controller, did: did:near:abcde.testnet, public key: "
    ///
    pub fn set_auth_key_by_controller(&mut self, did: String, pk: Vec<u8>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let controller_did = gen_did(&account_id);

        self.check_did_status(&did);
        self.check_did_status(&controller_did);
        let controller_list = self.controller.get(&did).unwrap();
        if !controller_list.contains(&controller_did) {
            env::panic(b"set_auth_key_by_controller, signer is not controller")
        }
        let controller_public_key_list = self.public_key.get(&controller_did).unwrap();
        controller_public_key_list.check_pk_access(&account_pk);

        let mut public_key_list = self.public_key.get(&did).unwrap();
        let index = public_key_list.set_pk_auth(&pk);
        self.public_key.insert(&did, &public_key_list);
        let mut authentication_list = self.authentication.get(&did).unwrap();
        authentication_list.push(index as u32);
        self.authentication.insert(&did, &authentication_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!(
            "set_auth_key_by_controller, did:{}, public key: {:?}",
            &did, pk
        );
        env::log(log_message.as_bytes());
    }

    /// deactivate_auth_key_by_controller
    ///
    /// this method will update the auth public key to invalid status. need the controller's signature.
    ///
    /// `pk` is controller's public key
    ///
    /// log information: "deactivate_auth_key_by_controller, did: did:near:abcde.testnet, public key: "
    ///
    pub fn deactivate_auth_key_by_controller(&mut self, did: String, pk: Vec<u8>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let controller_did = gen_did(&account_id);

        self.check_did_status(&did);
        self.check_did_status(&controller_did);
        let controller_list = self.controller.get(&did).unwrap();
        if !controller_list.contains(&controller_did) {
            env::panic(b"deactivate_auth_key_by_controller, signer is not controller")
        }
        let controller_public_key_list = self.public_key.get(&controller_did).unwrap();
        controller_public_key_list.check_pk_access(&account_pk);

        let mut public_key_list = self.public_key.get(&did).unwrap();
        let index = public_key_list.remove_pk_auth(&pk);
        self.public_key.insert(&did, &public_key_list);
        let mut authentication_list = self.authentication.get(&did).unwrap();
        let i = authentication_list
            .iter()
            .position(|x| x == &(index as u32))
            .unwrap();
        authentication_list.remove(i);
        self.authentication.insert(&did, &authentication_list);
        self.updated.insert(&did, &env::block_timestamp());

        let log_message = format!(
            "deactivate_auth_key_by_controller, did:{}, public key: {:?}",
            &did, pk
        );
        env::log(log_message.as_bytes());
    }

    /// add_service
    ///
    /// this method will add service to the DID.
    ///
    /// log information: "add_service, did: did:near:abcde.testnet, service id: 1234"
    ///
    pub fn add_service(&mut self, service_id: String, service_type: String, endpoint: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let ser = Service {
            id: service_id,
            tp: service_type,
            service_endpoint: endpoint,
        };
        let mut sers = self.service.get(&did).unwrap_or(vec![]);
        let index = sers.iter().position(|x| &x.id == &ser.id);
        let log_message = format!("add_service, did:{}, service id: {}", &did, &ser.id);
        if index.is_some() {
            env::panic(b"add_service, service exists")
        }
        sers.push(ser);
        self.service.insert(&did, &sers);
        self.updated.insert(&did, &env::block_timestamp());
        env::log(log_message.as_bytes());
    }

    /// update_service
    ///
    /// this method will update service.
    ///
    /// log information: "update_service, did: did:near:abcde.testnet, service id: 1234"
    ///
    pub fn update_service(&mut self, service_id: String, service_type: String, endpoint: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let ser = Service {
            id: service_id,
            tp: service_type,
            service_endpoint: endpoint,
        };
        let mut sers = self.service.get(&did).unwrap_or(vec![]);
        let index = sers.iter().position(|x| &x.id == &ser.id);
        let log_message = format!("update_service, did:{}, service id: {}", &did, &ser.id);
        match index {
            Some(ind) => {
                let res = sers.get_mut(ind).unwrap();
                res.id = ser.id;
                res.tp = ser.tp;
                res.service_endpoint = ser.service_endpoint;
                self.service.insert(&did, &sers);
            }
            _ => env::panic(b"update_service, service doesn't exist"),
        }
        self.updated.insert(&did, &env::block_timestamp());
        env::log(log_message.as_bytes());
    }

    /// remove_service
    ///
    /// this method will remove service.
    ///
    /// log information: "remove_service, did: did:near:abcde.testnet, service id: 1234"
    ///
    pub fn remove_service(&mut self, service_id: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let mut sers = self.service.get(&did).unwrap_or(vec![]);
        let index = sers.iter().position(|x| &x.id == &service_id);
        let log_message = format!("remove_service, did:{}, service id: {}", &did, &service_id);
        match index {
            Some(ind) => {
                sers.remove(ind);
                self.service.insert(&did, &sers);
            }
            _ => env::panic(b"remove_service, service doesn't exist"),
        }
        self.updated.insert(&did, &env::block_timestamp());
        env::log(log_message.as_bytes());
    }

    /// add_context
    ///
    /// this method will add context.
    ///
    /// log information: "add_context, did: did:near:abcde.testnet, context: 1234"
    ///
    pub fn add_context(&mut self, context: Vec<String>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let log_message = format!("add_context, did:{}, context: {:?}", &did, &context);
        let mut cons = self.contexts.get(&did).unwrap_or(vec![]);
        for v in context.iter() {
            if !cons.contains(v) && v != DEFAULT_CONTEXT1 && v != DEFAULT_CONTEXT2 {
                cons.push(v.clone());
            };
        }
        self.contexts.insert(&did, &cons);
        self.updated.insert(&did, &env::block_timestamp());
        env::log(log_message.as_bytes());
    }

    /// remove_context
    ///
    /// this method will remove context.
    ///
    /// log information: "remove_context, did: did:near:abcde.testnet, context: 1234"
    ///
    pub fn remove_context(&mut self, context: Vec<String>) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);

        let mut cons = self.contexts.get(&did).unwrap_or(vec![]);
        for v in context.iter() {
            let index = cons.iter().position(|x| x == v);
            if let Some(ind) = index {
                cons.remove(ind);
            }
        }

        self.updated.insert(&did, &env::block_timestamp());
        let log_message = format!("remove_context, did: {}, context: {:?}", &did, &context);
        env::log(log_message.as_bytes());
    }

    /// verify_signature
    ///
    /// this method will verify a transaction is signed by did athentication key.
    ///
    pub fn verify_signature(&self) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let did = gen_did(&account_id);

        self.check_did_status(&did);
        let public_key_list = self.public_key.get(&did).unwrap();
        public_key_list.check_pk_access(&account_pk);
    }

    /// verify_controller
    ///
    /// this method will verify a transaction is signed by did controller.
    ///
    /// this method will verify a DID is or not the controller.
    ///
    pub fn verify_controller(&self, did: String) {
        let account_id = env::signer_account_id();
        let account_pk = env::signer_account_pk();
        let controller_did = gen_did(&account_id);

        self.check_did_status(&did);
        self.check_did_status(&controller_did);
        let controller_list = self.controller.get(&did).unwrap();
        if !controller_list.contains(&controller_did) {
            env::panic(b"verify_controller, signer is not controller")
        }
        let controller_public_key_list = self.public_key.get(&controller_did).unwrap();
        controller_public_key_list.check_pk_access(&account_pk);
    }

    /// get_document
    ///
    /// this method query the did document, return json string.
    ///
    /// this method query the DID information.
    ///
    pub fn get_document(&self, did: String) -> Option<String> {
        let public_key_list = self.public_key.get(&did)?;
        let pk_list_json = public_key_list.get_pk_json(&did);
        let auth_index_list = self.authentication.get(&did)?;
        let authentication_list_json =
            public_key_list.get_authentication_json(&did, auth_index_list);
        let mut cts = self.contexts.get(&did).unwrap_or(vec![]);
        let mut contexts = vec![DEFAULT_CONTEXT1.to_string(), DEFAULT_CONTEXT2.to_string()];
        contexts.append(&mut cts);
        let mut service = self.service.get(&did).unwrap_or(vec![]);
        for v in service.iter_mut() {
            v.id = format!("{}#{}", &did, v.id);
        }
        let document = Document {
            contexts,
            public_key: pk_list_json,
            authentication: authentication_list_json,
            controller: self.controller.get(&did).unwrap_or(vec![]),
            service,
            created: self.created.get(&did).unwrap_or(0),
            updated: self.updated.get(&did).unwrap_or(0),
            id: did,
        };
        let document_json = serde_json::to_string(&document).unwrap_or("".to_string());
        Some(document_json)
    }

    fn check_did_status(&self, did: &String) {
        let status = self.status.get(did).unwrap();
        match status {
            Status::VALID => (),
            _ => env::panic(b"did status is not valid"),
        };
    }
}


#[derive(BorshDeserialize, BorshSerialize)]
pub enum Status {
    VALID = 0x00,
    DEACTIVATED = 0x01,
}

#[derive(Debug)]
pub enum KeyType {
    Ed25519VerificationKey2018,
    EcdsaSecp256k1VerificationKey2019,
}

impl KeyType {
    pub fn to_string(&self) -> String {
        match self {
            KeyType::Ed25519VerificationKey2018 => "Ed25519VerificationKey2018".to_string(),
            KeyType::EcdsaSecp256k1VerificationKey2019 => {
                "EcdsaSecp256k1VerificationKey2019".to_string()
            }
        }
    }
}

pub fn gen_did(account_id: &str) -> String {
    String::from("did:near:") + account_id
}

pub fn check_did(did: &str) {
    let head = &did[0..9];
    assert_eq!(head, "did:near:")
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PublicKey {
    controller: String,
    public_key: Vec<u8>,
    deactivated: bool,
    is_pk_list: bool,
    is_authentication: bool,
}

impl PublicKey {
    pub fn new_pk_and_auth(controller: &str, pk: Vec<u8>) -> Self {
        PublicKey {
            controller: controller.to_string(),
            public_key: pk,
            deactivated: false,
            is_pk_list: true,
            is_authentication: true,
        }
    }

    pub fn new_pk(controller: &str, pk: Vec<u8>) -> Self {
        PublicKey {
            controller: controller.to_string(),
            public_key: pk,
            deactivated: false,
            is_pk_list: true,
            is_authentication: false,
        }
    }

    pub fn new_auth(controller: &str, pk: Vec<u8>) -> Self {
        PublicKey {
            controller: controller.to_string(),
            public_key: pk,
            deactivated: false,
            is_pk_list: false,
            is_authentication: true,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PublicKeyList {
    public_key_list: Vec<PublicKey>,
}

impl PublicKeyList {
    pub fn new_default(controller: &str, pk: Vec<u8>) -> Self {
        PublicKeyList {
            public_key_list: vec![PublicKey::new_pk_and_auth(controller, pk)],
        }
    }

    pub fn push(&mut self, pk: PublicKey) {
        self.public_key_list.push(pk);
    }

    pub fn len(&self) -> u32 {
        self.public_key_list.len() as u32
    }

    pub fn pk_exist(&self, pk: &Vec<u8>) -> bool {
        for v in self.public_key_list.iter() {
            if &v.public_key == pk {
                return true;
            }
        }
        return false;
    }

    pub fn deactivate_pk(&mut self, pk: &Vec<u8>) {
        for v in self.public_key_list.iter_mut() {
            if &v.public_key == pk {
                if v.deactivated {
                    env::panic(b"deactivate_pk, pk is deactivated")
                }
                v.deactivated = true;
                return;
            }
        }
        env::panic(b"deactivate_pk, pk doesn't exist")
    }

    pub fn check_pk_access(&self, pk: &Vec<u8>) {
        for v in self.public_key_list.iter() {
            if &v.public_key == pk {
                if v.deactivated {
                    env::panic(b"check_pk_access, pk is deactivated")
                }
                if !v.is_authentication {
                    env::panic(b"check_pk_access, pk is not authentication")
                }
                return;
            }
        }
        env::panic(b"check_pk_access, pk doesn't exist")
    }

    pub fn set_pk_auth(&mut self, pk: &Vec<u8>) -> usize {
        for (index, v) in self.public_key_list.iter_mut().enumerate() {
            if &v.public_key == pk {
                if v.deactivated {
                    env::panic(b"set_pk_auth, pk is deactivated")
                }
                if v.is_authentication {
                    env::panic(b"set_pk_auth, pk is already auth key")
                }
                v.is_authentication = true;
                return index;
            }
        }
        env::panic(b"set_pk_auth, pk doesn't exist")
    }

    pub fn remove_pk_auth(&mut self, pk: &Vec<u8>) -> usize {
        for (index, v) in self.public_key_list.iter_mut().enumerate() {
            if &v.public_key == pk {
                if v.deactivated {
                    env::panic(b"remove_pk_auth, pk is deactivated")
                }
                if !v.is_authentication {
                    env::panic(b"remove_pk_auth, pk is not auth key")
                }
                v.is_authentication = false;
                return index;
            }
        }
        env::panic(b"remove_pk_auth, pk doesn't exist")
    }

    pub fn get_pk_json(&self, did: &str) -> Vec<PublicKeyJson> {
        let mut result = vec![];
        for (i, v) in self.public_key_list.iter().enumerate() {
            if !v.is_pk_list {
                continue;
            }
            let mut tp: String = "".to_string();
            match v.public_key[0] {
                0 => tp = KeyType::Ed25519VerificationKey2018.to_string(),
                1 => tp = KeyType::EcdsaSecp256k1VerificationKey2019.to_string(),
                _ => {}
            }
            let public_key_json = PublicKeyJson {
                id: format!("{}#keys-{}", did, i + 1),
                tp,
                controller: v.controller.clone(),
                public_key_base58: v.public_key.to_base58(),
            };
            result.push(public_key_json);
        }
        result
    }

    pub fn get_authentication_json(
        &self,
        did: &str,
        authentication_list: Vec<u32>,
    ) -> Vec<Authentication> {
        let mut result = vec![];
        for i in authentication_list.iter() {
            let public_key: &PublicKey = self.public_key_list.get(*i as usize).unwrap();
            if public_key.is_pk_list {
                let authentication = Authentication::Pk(format!("{}#keys-{}", did, i + 1));
                result.push(authentication);
            } else {
                let mut tp: String = "".to_string();
                match public_key.public_key[0] {
                    0 => tp = KeyType::Ed25519VerificationKey2018.to_string(),
                    1 => tp = KeyType::EcdsaSecp256k1VerificationKey2019.to_string(),
                    _ => {}
                }
                let authentication = Authentication::NotPK(PublicKeyJson {
                    id: format!("{}#keys-{}", did, i + 1),
                    tp,
                    controller: public_key.controller.clone(),
                    public_key_base58: public_key.public_key.to_base58(),
                });
                result.push(authentication);
            }
        }
        result
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PublicKeyJson {
    id: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    tp: String,
    controller: String,
    #[serde(rename(serialize = "publicKeyBase58", deserialize = "publicKeyBase58"))]
    public_key_base58: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Service {
    pub id: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub tp: String,
    #[serde(rename(serialize = "serviceEndpoint", deserialize = "serviceEndpoint"))]
    pub service_endpoint: String,
}

#[cfg(test)]
impl Service {
    pub fn new(id: String, tp: String, service_endpoint: String) -> Self {
        Service {
            id,
            tp,
            service_endpoint,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Authentication {
    Pk(String),
    NotPK(PublicKeyJson),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename(serialize = "@contexts", deserialize = "@contexts"))]
    pub contexts: Vec<String>,
    pub id: String,
    #[serde(rename(serialize = "publicKey", deserialize = "publicKey"))]
    pub public_key: Vec<PublicKeyJson>,
    pub authentication: Vec<Authentication>,
    pub controller: Vec<String>,
    pub service: Vec<Service>,
    pub created: u64,
    pub updated: u64,
}

