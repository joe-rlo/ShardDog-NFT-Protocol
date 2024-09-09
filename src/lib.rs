use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, Promise};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde_json;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub channels: UnorderedMap<String, Channel>,
    pub minted_tokens: UnorderedSet<TokenId>,
    pub metadata: NFTContractMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Channel {
    pub merkle_root: Vec<u8>,
    pub total_supply: u64,
    pub next_token_number: u64,
    pub metadata: ChannelMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ChannelMetadata {
    pub title: String,
    pub description: String,
    pub media: String,
    pub animation_url: Option<String>,
    pub reference: String,
    pub reference_hash: Option<Base64VecU8>,
    
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub base_uri: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
}

pub type TokenId = String; // Format: "{channel_id}:{token_number}"

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            channels: UnorderedMap::new(b"c"),
            minted_tokens: UnorderedSet::new(b"m"),
            metadata: NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "NFT Collection".to_string(),
                symbol: "NFT".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        }
    }

    pub fn create_channel(&mut self, channel_id: String, merkle_root: Vec<u8>, metadata: ChannelMetadata) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the owner can create channels");
        let channel = Channel {
            merkle_root,
            total_supply: 0,
            next_token_number: 1,
            metadata: ChannelMetadata {
                title: metadata.title.clone(),
                description: metadata.description.clone(),
                media: metadata.media.clone(),
                animation_url: metadata.animation_url.clone(),
                reference: metadata.reference.clone(),
                reference_hash: metadata.reference_hash.clone(), // Clone here
            },
        };
        self.channels.insert(&channel_id, &channel);
    
        // Log the creation of the new channel
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"create_series\",\"data\":{{\"id\":\"{}\",\"metadata\":{}}}}}",
            channel_id,
            serde_json::to_string(&metadata).unwrap()
        ));
    }


    #[payable]
    pub fn mint(&mut self, channel_id: String, proof: Vec<Vec<u8>>) -> Promise {
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
        let token_number = channel.next_token_number;
        let token_id = format!("{}:{}", channel_id, token_number);
        
        assert!(!self.minted_tokens.contains(&token_id), "Token already minted");
    
        assert!(
            self.verify_merkle_proof(&channel.merkle_root, &token_id, &proof),
            "Invalid Merkle proof"
        );
    
        let owner_id = env::predecessor_account_id();
        self.minted_tokens.insert(&token_id);
    
        channel.total_supply += 1;
        channel.next_token_number += 1;
        self.channels.insert(&channel_id, &channel);
    
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_mint\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            owner_id, token_id
        ));
    
        Promise::new(env::current_account_id())
    }

    pub fn is_minted(&self, token_id: TokenId) -> bool {
        self.minted_tokens.contains(&token_id)
    }

    pub fn get_next_token_number(&self, channel_id: String) -> Option<u64> {
        self.channels.get(&channel_id).map(|channel| channel.next_token_number)
    }

    pub fn get_channel_info(&self, channel_id: String) -> Option<Channel> {
        self.channels.get(&channel_id)
    }

    pub fn update_merkle_root(&mut self, channel_id: String, new_merkle_root: Vec<u8>) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the owner can update the Merkle root");
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
        channel.merkle_root = new_merkle_root;
        self.channels.insert(&channel_id, &channel);
    }

    fn verify_merkle_proof(&self, root: &[u8], token_id: &str, proof: &Vec<Vec<u8>>) -> bool {
        let mut computed_hash = env::sha256(token_id.as_bytes());
        for proof_element in proof.iter() {
            let mut combined = Vec::with_capacity(computed_hash.len() + proof_element.len());
            if computed_hash <= *proof_element {
                combined.extend_from_slice(&computed_hash);
                combined.extend_from_slice(proof_element);
            } else {
                combined.extend_from_slice(proof_element);
                combined.extend_from_slice(&computed_hash);
            }
            computed_hash = env::sha256(&combined);
        }
        computed_hash.to_vec() == root
    }

    #[payable]
    pub fn transfer(&mut self, token_id: TokenId, receiver_id: AccountId) {
        let sender_id = env::predecessor_account_id();
        let (channel_id, _) = token_id.split_once(':').expect("Invalid token ID format");
        
        assert!(self.minted_tokens.contains(&token_id), "Token does not exist");
        
        // Implement your transfer logic here
        // For simplicity, we're not checking ownership, but you should in a real implementation
        
        // Log the transfer event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_transfer\",\"data\":[{{\"old_owner_id\":\"{}\",\"new_owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            sender_id, receiver_id, token_id
        ));
    }

    #[payable]
    pub fn burn(&mut self, token_id: TokenId, new_merkle_root: Vec<u8>) {
        let sender_id = env::predecessor_account_id();
        let (channel_id, token_number_str) = token_id.split_once(':').expect("Invalid token ID format");
        let token_number: u64 = token_number_str.parse().expect("Invalid token number");
        
        assert!(self.minted_tokens.contains(&token_id), "Token does not exist");
        
        // Remove the token from minted_tokens
        self.minted_tokens.remove(&token_id);
        
        // Update the channel's total supply and Merkle root
        let mut channel = self.channels.get(&channel_id.to_string()).expect("Channel not found");
        channel.total_supply -= 1;
        channel.merkle_root = new_merkle_root;
        self.channels.insert(&channel_id.to_string(), &channel);
        
        // Log the burn event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_burn\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            sender_id, token_id
    ));
}
}