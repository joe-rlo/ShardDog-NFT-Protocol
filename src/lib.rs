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

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: String,
    pub description: String,
    pub media: String,
    pub animation_url: Option<String>,
    pub reference: String,
    pub reference_hash: Option<Base64VecU8>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: String,
    pub metadata: TokenMetadata,
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
        let contract = Self {
            owner_id: owner_id.clone(),
            channels: UnorderedMap::new(b"c"),
            minted_tokens: UnorderedSet::new(b"m"),
            metadata: NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "ShardDog".to_string(),
                symbol: "SHARDDOG".to_string(),
                icon: Some("data:image/png;base64,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".to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        };
         // Log contract initialization
            env::log_str(&format!(
                "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"init\",\"data\":{{\"owner_id\":\"{}\"}}}}",
                owner_id
            ));

            contract
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
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"create_series\",\"data\":{{\"series_id\":\"{}\",\"metadata\":{}}}}}",
            channel_id,
            serde_json::to_string(&metadata).unwrap()
        ));
    }

    pub fn nft_token(&self, token_id: String) -> Option<JsonToken> {
        let parts: Vec<&str> = token_id.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
    
        let channel_id = parts[0];
        let token_number: u64 = parts[1].parse().ok()?;
    
        if !self.minted_tokens.contains(&token_id) {
            return None;
        }
    
        let channel = self.channels.get(&channel_id.to_string())?;
    
        let token_metadata = TokenMetadata {
            title: format!("{} #{}", channel.metadata.title, token_number),
            description: channel.metadata.description,
            media: channel.metadata.media,
            animation_url: channel.metadata.animation_url,
            reference: format!("{}/{}", channel.metadata.reference, token_number),
            reference_hash: channel.metadata.reference_hash,
        };
    
        Some(JsonToken {
            token_id,
            metadata: token_metadata,
        })
    }


    pub fn nft_mint(&mut self, channel_id: String, proof: Option<Vec<Vec<u8>>>, receiver_id: AccountId) -> (TokenId, serde_json::Value) {
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
        let token_number = channel.next_token_number;
        let token_id = format!("{}:{}", channel_id, token_number);
        
        assert!(!self.minted_tokens.contains(&token_id), "Token already minted");
    
        if token_number == 1 {
            // This is the first token, no proof needed
            if proof.is_some() {
                env::panic_str("First token should be minted without a proof");
            }
            // Establish the initial Merkle root
            let initial_root = env::sha256(token_id.as_bytes());
            channel.merkle_root = initial_root.to_vec();
        } else {
            // For subsequent tokens, require and verify the proof
            let proof = proof.expect("Proof is required for non-first tokens");
            assert!(
                self.verify_merkle_proof(&channel.merkle_root, &token_id, &proof),
                "Invalid Merkle proof"
            );
        }
    
        let owner_id = receiver_id;
        self.minted_tokens.insert(&token_id);
    
        channel.total_supply += 1;
        channel.next_token_number += 1;
        self.channels.insert(&channel_id, &channel);
    
        let token_metadata = serde_json::json!({
            "title": format!("{} #{}", channel.metadata.title, token_number),
            "description": channel.metadata.description,
            "media": channel.metadata.media,
            "animation_url": channel.metadata.animation_url,
            "media_hash": null, // Add if you have a media hash
            "issued_at": env::block_timestamp().to_string(),
            "reference": channel.metadata.reference,
            "reference_hash": channel.metadata.reference_hash
        });
    
          // Log the minting event in the standard NEP-171 format
          env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_mint\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"],\"memo\":null,\"metadata\":{}}}]}}",
            owner_id, token_id, token_metadata.to_string()
        ));
    

        (token_id, token_metadata)
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
       // env::log_str(&format!("Initial hash: {:?}", computed_hash));
        
        for (i, proof_element) in proof.iter().enumerate() {
            let mut combined = Vec::with_capacity(64);
            if computed_hash <= *proof_element {
                combined.extend_from_slice(&computed_hash);
                combined.extend_from_slice(proof_element);
            } else {
                combined.extend_from_slice(proof_element);
                combined.extend_from_slice(&computed_hash);
            }
            computed_hash = env::sha256(&combined);
           // env::log_str(&format!("After step {}: {:?}", i, computed_hash));
        }
        
        //env::log_str(&format!("Final hash: {:?}", computed_hash));
        //env::log_str(&format!("Root: {:?}", root));
        
        computed_hash == root
    }

    #[payable]
    pub fn nft_transfer(&mut self, token_id: TokenId, receiver_id: AccountId) {
        let sender_id = env::predecessor_account_id();
        let (channel_id, _) = token_id.split_once(':').expect("Invalid token ID format");
        
        assert!(self.minted_tokens.contains(&token_id), "Token does not exist");
        
        //Logic is handled off-chain before this is called.
        
        // Log the transfer event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_transfer\",\"data\":[{{\"old_owner_id\":\"{}\",\"new_owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            sender_id, receiver_id, token_id
        ));
    }

    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId, new_merkle_root: Vec<u8>) {
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