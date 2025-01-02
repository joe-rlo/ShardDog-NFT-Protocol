use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, BorshStorageKey};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use std::clone::Clone;
use near_sdk::{Promise, assert_one_yocto};
use near_sdk::NearToken;

pub const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    Channels,
    TokenIndex,
    MintedTokens,
    Owners,
    OwnerTokens { account_id: AccountId },
    ChannelIndex,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub channels: UnorderedMap<String, Channel>,
    pub minted_tokens: UnorderedSet<TokenId>,
    pub metadata: NFTContractMetadata,
    pub owners: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub channel_index: UnorderedMap<u16, String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Channel {
    pub merkle_root: Vec<u8>,
    pub total_possible: u64,    
    pub minted_tokens: UnorderedSet<u64>,
    pub total_supply: u64,
    pub next_token_number: u64,
    pub metadata: ChannelMetadata,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ChannelView {
    pub merkle_root: Vec<u8>,
    pub total_possible: u64,    
    pub total_supply: u64,
    pub next_token_number: u64,
    pub metadata: ChannelMetadata,
}

impl From<Channel> for ChannelView {
    fn from(channel: Channel) -> Self {
        ChannelView {
            merkle_root: channel.merkle_root,
            total_possible: channel.total_possible,
            total_supply: channel.total_supply,
            next_token_number: channel.next_token_number,
            metadata: channel.metadata,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ChannelMetadata {
    pub title_template: String,
    pub description_template: String,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
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
            channels: UnorderedMap::new(StorageKey::Channels),
            minted_tokens: UnorderedSet::new(StorageKey::MintedTokens),
            metadata: NFTContractMetadata {
                spec: "nft-2.1.0".to_string(),
                name: "ShardDog".to_string(),
                symbol: "SHARDDOG".to_string(),
                icon: Some("https://arweave.net/U8NfqCM4-OIqbCkA8fXT1a5LVkd5eNb1jO5A-8Q4oEY".to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
            owners: LookupMap::new(StorageKey::Owners),
            channel_index: UnorderedMap::new(StorageKey::ChannelIndex),
        };
        contract
    }

    pub fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.clone()
    }

    pub fn create_channel(&mut self, channel_id: String, merkle_root: Vec<u8>, metadata: ChannelMetadata) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the owner can create channels");
        let channel = Channel {
            merkle_root,
            total_possible: u64::MAX,    // Set to maximum possible value
            minted_tokens: UnorderedSet::new(b"m"),
            total_supply: 0,
            next_token_number: 1,
            metadata: ChannelMetadata {
                title_template: metadata.title_template.clone(),
                description_template: metadata.description_template.clone(),
                media: metadata.media.clone(),
                animation_url: metadata.animation_url.clone(),
                reference: metadata.reference.clone(),
                reference_hash: metadata.reference_hash.clone(),
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

    pub fn update_channel(&mut self, channel_id: String, merkle_root: Option<Vec<u8>>, metadata: Option<ChannelMetadata>) {
        // Only the contract owner can update channels
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the owner can update channels");
    
        // Get the existing channel
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
    
        // Update merkle root if provided
        if let Some(new_merkle_root) = merkle_root {
            channel.merkle_root = new_merkle_root;
        }
    
        // Update metadata if provided
        if let Some(new_metadata) = metadata {
            channel.metadata = ChannelMetadata {
                title_template: new_metadata.title_template,
                description_template: new_metadata.description_template,
                media: new_metadata.media,
                animation_url: new_metadata.animation_url,
                reference: new_metadata.reference,
                reference_hash: new_metadata.reference_hash,
            };
        }
    
        // Save the updated channel
        self.channels.insert(&channel_id, &channel);
    
        // Log the update event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"update_series\",\"data\":{{\"series_id\":\"{}\",\"metadata\":{}}}}}",
            channel_id,
            serde_json::to_string(&channel.metadata).unwrap()
        ));
    }

    pub fn nft_token(&self, token_id: String) -> Option<JsonToken> {
        let parts: Vec<&str> = token_id.split(':').collect();
        if parts.len() != 2 {
            return None;
        }
    
        let channel_id = parts[0];
        let token_number: u64 = parts[1].parse().ok()?;
    
        let channel = self.channels.get(&channel_id.to_string())?;
    
        let token_metadata = TokenMetadata {
            title: channel.metadata.title_template.replace("{}", &token_number.to_string()),
            description: channel.metadata.description_template.clone(),
            media: channel.metadata.media.clone(),
            animation_url: channel.metadata.animation_url.clone(),
            reference: format!("{}/{}", channel.metadata.reference, token_number),
            reference_hash: channel.metadata.reference_hash.clone(),
        };
    
        Some(JsonToken {
            token_id,
            metadata: token_metadata,
        })
    }


    #[payable]
    pub fn nft_mint(
        &mut self,
        channel_id: String,
        proof: Option<Vec<Vec<u8>>>,
        receiver_id: AccountId,
    ) -> TokenId {
        let initial_storage = env::storage_usage();
        
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
        let token_number = channel.next_token_number;
        
        // Remove total_possible check since we want unlimited minting
        
        // Verify proof for non-first tokens
        if token_number > 1 {
            let proof = proof.expect("Proof required for minting");
            let token_id = format!("{}:{}", channel_id, token_number);
            assert!(
                self.verify_merkle_proof(&channel.merkle_root, &token_id, &proof),
                "Invalid proof"
            );
        }

        let token_id = format!("{}:{}", channel_id, token_number);
        
        // Update states
        channel.minted_tokens.insert(&token_number);
        channel.next_token_number += 1;
        channel.total_supply += 1;
        self.channels.insert(&channel_id, &channel);
        
        self.minted_tokens.insert(&token_id);
        let mut owner_tokens = self.owners
            .get(&receiver_id)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens {
                account_id: receiver_id.clone(),
            }));
        owner_tokens.insert(&token_id);
        self.owners.insert(&receiver_id, &owner_tokens);

        // Verify sufficient deposit
        let required_storage = env::storage_usage() - initial_storage;
        let required_cost = env::storage_byte_cost().saturating_mul(required_storage as u128);
        
        assert!(
            env::attached_deposit() >= required_cost,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost,
        );

        // Emit NEP-171 event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_mint\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            receiver_id,
            token_id
        ));

        token_id
    }


    pub fn is_minted(&self, token_id: TokenId) -> bool {
        self.minted_tokens.contains(&token_id)
    }

    pub fn get_next_token_number(&self, channel_id: String) -> Option<u64> {
        self.channels.get(&channel_id).map(|channel| channel.next_token_number)
    }

    pub fn get_channel_info(&self, channel_id: String) -> Option<ChannelView> {
        self.channels.get(&channel_id).map(|channel| channel.into())
    }
    

    pub fn update_merkle_root(&mut self, channel_id: String, new_merkle_root: Vec<u8>) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only the owner can update the Merkle root");
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
        channel.merkle_root = new_merkle_root;
        self.channels.insert(&channel_id, &channel);
    }

    fn verify_merkle_proof(
        &self,
        root: &[u8],
        token_id: &str,
        proof: &Vec<Vec<u8>>,
    ) -> bool {
        let mut hash = env::sha256(token_id.as_bytes());
        for proof_element in proof {
            let combined = if hash <= *proof_element {
                [hash.as_slice(), proof_element].concat()
            } else {
                [proof_element, hash.as_slice()].concat()
            };
            hash = env::sha256(&combined);
        }
        hash == root
    }

    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,  // Added for NEP-171 compatibility
        memo: Option<String>       // Added for NEP-171 compatibility
    ) {
        assert_one_yocto();  // Require attached deposit of exactly 1 yoctoNEAR
        let sender_id = env::predecessor_account_id();
        
        // Get sender's tokens
        let mut sender_tokens = self.owners
            .get(&sender_id)
            .expect("Sender does not own this token");
        
        // Check if sender owns the specific token
        assert!(
            sender_tokens.contains(&token_id),
            "Sender does not own this token"
        );

        // Prevent transferring to the same account
        assert_ne!(
            &sender_id, &receiver_id,
            "The token owner and the receiver should be different"
        );

        // Remove token from sender
        sender_tokens.remove(&token_id);
        
        // Update or remove sender's token set
        if sender_tokens.is_empty() {
            self.owners.remove(&sender_id);
        } else {
            self.owners.insert(&sender_id, &sender_tokens);
        }

        // Add token to receiver
        let mut receiver_tokens = self.owners
            .get(&receiver_id)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens { 
                account_id: receiver_id.clone() 
            }));
        
        receiver_tokens.insert(&token_id);
        self.owners.insert(&receiver_id, &receiver_tokens);

        // Log the transfer with memo if provided
        let memo_str = memo.unwrap_or_default();
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_transfer\",\"data\":[{{\"authorized_id\":null,\"old_owner_id\":\"{}\",\"new_owner_id\":\"{}\",\"token_ids\":[\"{}\"],\"memo\":\"{}\"}}]}}",
            sender_id,
            receiver_id,
            token_id,
            memo_str
        ));
    }

    #[payable]
    pub fn nft_burn(&mut self, token_id: TokenId) {
        let owner_id = env::predecessor_account_id();
        
        // Verify ownership
        let mut owner_tokens = self.owners.get(&owner_id).expect("Owner has no tokens");
        assert!(owner_tokens.contains(&token_id), "Sender does not own this token");

        // Remove token from owner
        owner_tokens.remove(&token_id);
        if owner_tokens.is_empty() {
            self.owners.remove(&owner_id);
        } else {
            self.owners.insert(&owner_id, &owner_tokens);
        }

        // Remove from minted_tokens
        self.minted_tokens.remove(&token_id);

        // Update channel data
        let (channel_id, _) = token_id.split_once(':').unwrap();
        let mut channel = self.channels.get(&channel_id.to_string()).expect("Channel not found");
        channel.total_supply -= 1;
        self.channels.insert(&channel_id.to_string(), &channel);

        // Emit burn event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_burn\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            owner_id, token_id
        ));
    }

    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>
    ) -> Vec<JsonToken> {
        // Get the set of tokens for the given account, or create an empty set if none exists
        let tokens = self.owners.get(&account_id)
            .unwrap_or_else(|| UnorderedSet::new(StorageKey::OwnerTokens { 
                account_id: account_id.clone() 
            }));
        
        // Convert from_index from U128 to usize
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let limit = limit.unwrap_or(50) as usize;  // Default limit of 50 tokens

        // Get the tokens with pagination
        tokens.iter()
            .skip(start)
            .take(limit)
            .filter_map(|token_id| self.nft_token(token_id.clone()))
            .collect()
    }

    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> near_sdk::json_types::U128 {
        near_sdk::json_types::U128(
            self.owners
                .get(&account_id)
                .map(|tokens| tokens.len() as u128)
                .unwrap_or(0)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new(accounts(1));
        assert_eq!(contract.owner_id, accounts(1));
    }

    #[test]
    fn test_create_channel() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1));

        let metadata = ChannelMetadata {
            title_template: "Test Channel #{}".to_string(),
            description_template: "Test Description".to_string(),
            media: "https://example.com/image.jpg".to_string(),
            animation_url: None,
            reference: "https://example.com/ref".to_string(),
            reference_hash: None,
        };

        contract.create_channel(
            "test_channel".to_string(),
            vec![1, 2, 3],  // Example merkle root
            metadata.clone()
        );

        let channel = contract.get_channel_info("test_channel".to_string()).unwrap();
        assert_eq!(channel.total_supply, 0);
        assert_eq!(channel.next_token_number, 1);
        assert_eq!(channel.metadata.title_template, "Test Channel #{}");
    }

    #[test]
    fn test_mint_and_transfer() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1));

        // Create channel
        let metadata = ChannelMetadata {
            title_template: "Test Channel #{}".to_string(),
            description_template: "Test Description".to_string(),
            media: "https://example.com/image.jpg".to_string(),
            animation_url: None,
            reference: "https://example.com/ref".to_string(),
            reference_hash: None,
        };

        contract.create_channel(
            "test_channel".to_string(),
            vec![1, 2, 3],
            metadata
        );

        // Mint token
        let (token_id, _) = contract.nft_mint(
            "test_channel".to_string(),
            None,  // First token doesn't need proof
            accounts(2)
        );

        // Check ownership
        let owner_tokens = contract.nft_tokens_for_owner(accounts(2), None, None);
        assert_eq!(owner_tokens.len(), 1);
        assert_eq!(owner_tokens[0].token_id, token_id);

        // Transfer token
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .build());
        contract.nft_transfer(token_id.clone(), accounts(3));

        // Check new ownership
        let new_owner_tokens = contract.nft_tokens_for_owner(accounts(3), None, None);
        assert_eq!(new_owner_tokens.len(), 1);
        assert_eq!(new_owner_tokens[0].token_id, token_id);
    }

    #[test]
    fn test_burn() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1));

        // Create channel and mint token
        let metadata = ChannelMetadata {
            title_template: "Test Channel #{}".to_string(),
            description_template: "Test Description".to_string(),
            media: "https://example.com/image.jpg".to_string(),
            animation_url: None,
            reference: "https://example.com/ref".to_string(),
            reference_hash: None,
        };

        contract.create_channel(
            "test_channel".to_string(),
            vec![1, 2, 3],
            metadata
        );

        let (token_id, _) = contract.nft_mint(
            "test_channel".to_string(),
            None,
            accounts(2)
        );

        // Burn token
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .build());
        contract.nft_burn(token_id.clone());

        // Verify token is burned
        assert!(!contract.is_minted(token_id));
        let owner_tokens = contract.nft_tokens_for_owner(accounts(2), None, None);
        assert_eq!(owner_tokens.len(), 0);
    }

    #[test]
    #[should_panic(expected = "Only the owner can create channels")]
    fn test_create_channel_not_owner() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1));

        let metadata = ChannelMetadata {
            title_template: "Test Channel #{}".to_string(),
            description_template: "Test Description".to_string(),
            media: "https://example.com/image.jpg".to_string(),
            animation_url: None,
            reference: "https://example.com/ref".to_string(),
            reference_hash: None,
        };

        contract.create_channel(
            "test_channel".to_string(),
            vec![1, 2, 3],
            metadata
        );
    }

    #[test]
    #[should_panic(expected = "Sender does not own this token")]
    fn test_transfer_not_owner() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let mut contract = Contract::new(accounts(1));

        // Create channel and mint token
        let metadata = ChannelMetadata {
            title_template: "Test Channel #{}".to_string(),
            description_template: "Test Description".to_string(),
            media: "https://example.com/image.jpg".to_string(),
            animation_url: None,
            reference: "https://example.com/ref".to_string(),
            reference_hash: None,
        };

        contract.create_channel(
            "test_channel".to_string(),
            vec![1, 2, 3],
            metadata
        );

        let (token_id, _) = contract.nft_mint(
            "test_channel".to_string(),
            None,
            accounts(2)
        );

        // Try to transfer token from wrong account
        testing_env!(context
            .predecessor_account_id(accounts(3))
            .build());
        contract.nft_transfer(token_id, accounts(4));
    }
}