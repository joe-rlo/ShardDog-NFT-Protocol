use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, LookupMap, LookupSet};
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, BorshStorageKey};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json;
use std::clone::Clone;
use near_sdk::{Promise, assert_one_yocto};
use near_sdk::NearToken;
use borsh::to_vec;

pub const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Owners,
    TokenOwners,
    Minted,
    Channels,
    TokenToChannel,
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

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct Channel {
    pub merkle_root: Vec<u8>,
    pub next_token_number: u64,
    pub total_supply: u64,
    pub metadata: ChannelMetadata,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub token_id: String,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub media_hash: Option<Base64VecU8>,
    pub copies: Option<u64>,
    pub issued_at: Option<String>,
    pub expires_at: Option<String>,
    pub starts_at: Option<String>,
    pub updated_at: Option<String>,
    pub extra: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub minted_count: u64,
    pub owners: LookupMap<AccountId, Vec<u64>>,
    pub token_owners: LookupMap<u64, AccountId>,
    pub minted_tokens: LookupSet<u64>,
    pub channels: LookupMap<String, Channel>,
    pub token_to_channel: LookupMap<u64, String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OwnerTokens {
    pub channel_tokens: UnorderedMap<String, Vec<u64>>, // channel_id -> vec of token numbers
}

impl OwnerTokens {
    pub fn new(prefix: Vec<u8>) -> Self {
        Self {
            channel_tokens: UnorderedMap::new(prefix),
        }
    }

    pub fn add_token(&mut self, channel_id: String, token_number: u64) {
        let mut numbers = self.channel_tokens.get(&channel_id)
            .unwrap_or_default();
        numbers.push(token_number);
        self.channel_tokens.insert(&channel_id, &numbers);
    }

    pub fn contains_token(&self, channel_id: String, token_number: u64) -> bool {
        self.channel_tokens.get(&channel_id)
            .map(|numbers| numbers.contains(&token_number))
            .unwrap_or(false)
    }

    pub fn remove_token(&mut self, channel_id: String, token_number: u64) -> bool {
        if let Some(mut numbers) = self.channel_tokens.get(&channel_id) {
            if let Some(pos) = numbers.iter().position(|&x| x == token_number) {
                numbers.swap_remove(pos);
                if numbers.is_empty() {
                    self.channel_tokens.remove(&channel_id);
                } else {
                    self.channel_tokens.insert(&channel_id, &numbers);
                }
                return true;
            }
        }
        false
    }

    pub fn get_all_tokens(&self) -> Vec<String> {
        let mut tokens = Vec::new();
        for (channel_id, numbers) in self.channel_tokens.iter() {
            for &number in numbers.iter() {
                tokens.push(format!("{}:{}", channel_id, number));
            }
        }
        tokens
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
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
            total_possible: u64::MAX,    // Set to maximum possible value
            total_supply: channel.total_supply,
            next_token_number: channel.next_token_number,
            metadata: channel.metadata,
        }
    }
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
        Self {
            owner_id,
            minted_count: 0,
            owners: LookupMap::new(StorageKey::Owners),
            token_owners: LookupMap::new(StorageKey::TokenOwners),
            minted_tokens: LookupSet::new(StorageKey::Minted),
            channels: LookupMap::new(StorageKey::Channels),
            token_to_channel: LookupMap::new(StorageKey::TokenToChannel),
        }
    }

    pub fn nft_metadata(&self) -> NFTContractMetadata {
        NFTContractMetadata {
            spec: "nft-2.1.0".to_string(),
            name: "ShardDog".to_string(),
            symbol: "SHARDDOG".to_string(),
            icon: Some("https://arweave.net/U8NfqCM4-OIqbCkA8fXT1a5LVkd5eNb1jO5A-8Q4oEY".to_string()),
            base_uri: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[payable]
    pub fn create_channel(
        &mut self, 
        channel_id: String, 
        merkle_root: Vec<u8>,
        metadata: ChannelMetadata,
    ) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner_id,
            "Only owner can create channels"
        );
        
        assert!(
            !self.channels.contains_key(&channel_id),
            "Channel already exists"
        );

        let channel = Channel {
            merkle_root,
            next_token_number: 1,
            total_supply: 0,
            metadata: metadata.clone(),
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
            let updated_channel = Channel {
                merkle_root: channel.merkle_root,
                next_token_number: channel.next_token_number,
                total_supply: channel.total_supply,
                metadata: new_metadata,
            };
            self.channels.insert(&channel_id, &updated_channel);
        }
    
        // Log the update event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"update_series\",\"data\":{{\"series_id\":\"{}\",\"metadata\":{}}}}}",
            channel_id,
            serde_json::to_string(&self.channels.get(&channel_id).expect("Channel not found").metadata).unwrap()
        ));
    }

    pub fn nft_token(&self, token_id: String) -> Option<JsonToken> {
        let token_id: u64 = token_id.parse().ok()?;
        let owner_id = self.token_owners.get(&token_id)?;
        let channel_id = self.token_to_channel.get(&token_id)?;
        let channel = self.channels.get(&channel_id)?;

        Some(JsonToken {
            token_id: token_id.to_string(),
            owner_id,
            metadata: TokenMetadata {
                title: Some(channel.metadata.title_template.replace("{}", &token_id.to_string())),
                description: Some(channel.metadata.description_template.clone()),
                media: Some(channel.metadata.media.clone()),
                media_hash: None,
                copies: None,
                issued_at: None,
                expires_at: None,
                starts_at: None,
                updated_at: None,
                extra: None,
                reference: Some(format!("{}/{}", channel.metadata.reference, token_id)),
                reference_hash: channel.metadata.reference_hash.clone(),
            },
        })
    }

    #[payable]
    pub fn nft_mint(
        &mut self,
        receiver_id: AccountId,
        channel_id: String,
        proof: Option<Vec<Vec<u8>>>,
    ) -> u64 {
        let initial_storage = env::storage_usage();
        
        let mut channel = self.channels.get(&channel_id).expect("Channel not found");
        let token_number = channel.next_token_number;

        // Verify proof for non-first tokens
        if token_number > 1 {
            let proof = proof.expect("Proof required for minting");
            let token_str = format!("{}:{}", channel_id, token_number);
            assert!(
                self.verify_merkle_proof(&channel.merkle_root, &token_str, &proof),
                "Invalid proof"
            );
        }

        // Update channel state
        channel.next_token_number += 1;
        channel.total_supply += 1;
        self.channels.insert(&channel_id, &channel);

        // Increment and get new token ID
        self.minted_count += 1;
        let token_id = self.minted_count;

        // Store token data
        self.minted_tokens.insert(&token_id);
        self.token_to_channel.insert(&token_id, &channel_id);

        // Store ownership data
        let mut tokens = self.owners.get(&receiver_id).unwrap_or_default();
        tokens.push(token_id);
        self.owners.insert(&receiver_id, &tokens);
        self.token_owners.insert(&token_id, &receiver_id);

        // Calculate required deposit
        let required_storage = env::storage_usage() - initial_storage;
        let required_cost = env::storage_byte_cost().saturating_mul(required_storage as u128);
        
        assert!(
            env::attached_deposit() >= required_cost,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost,
        );

        // Emit mint event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_mint\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            receiver_id,
            token_id
        ));

        token_id
    }

    pub fn is_minted(&self, token_id: u64) -> bool {
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
        merkle_root: &[u8],
        token_id: &str,
        proof: &[Vec<u8>],
    ) -> bool {
        let mut current = env::sha256(token_id.as_bytes());
        
        for proof_item in proof {
            let mut combined = Vec::with_capacity(64);
            if proof_item < &current {
                combined.extend_from_slice(proof_item);
                combined.extend_from_slice(&current);
            } else {
                combined.extend_from_slice(&current);
                combined.extend_from_slice(proof_item);
            }
            current = env::sha256(&combined);
        }
        
        &current == merkle_root
    }

    #[payable]
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        assert_one_yocto();
        let token_id: u64 = token_id.parse().expect("Invalid token ID");
        let sender_id = env::predecessor_account_id();
        
        // Verify ownership
        assert_eq!(
            self.token_owners.get(&token_id).expect("Token not found"),
            sender_id,
            "Sender must own token"
        );

        // Remove from sender
        let mut sender_tokens = self.owners.get(&sender_id).expect("No tokens found");
        sender_tokens.retain(|&x| x != token_id);
        
        if sender_tokens.is_empty() {
            self.owners.remove(&sender_id);
        } else {
            self.owners.insert(&sender_id, &sender_tokens);
        }

        // Add to receiver
        let mut receiver_tokens = self.owners.get(&receiver_id).unwrap_or_default();
        receiver_tokens.push(token_id);
        self.owners.insert(&receiver_id, &receiver_tokens);
        self.token_owners.insert(&token_id, &receiver_id);

        // Emit transfer event
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
    pub fn nft_burn(&mut self, token_id: String) {
        assert_one_yocto();
        let token_id: u64 = token_id.parse().expect("Invalid token ID");
        let owner_id = env::predecessor_account_id();

        // Verify ownership
        assert_eq!(
            self.token_owners.get(&token_id).expect("Token not found"),
            owner_id,
            "Must own token to burn it"
        );

        // Remove from owner's tokens
        let mut owner_tokens = self.owners.get(&owner_id).expect("No tokens found");
        owner_tokens.retain(|&x| x != token_id);
        
        if owner_tokens.is_empty() {
            self.owners.remove(&owner_id);
        } else {
            self.owners.insert(&owner_id, &owner_tokens);
        }

        // Remove from token ownership tracking
        self.token_owners.remove(&token_id);
        self.minted_tokens.remove(&token_id);

        // Emit burn event
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_burn\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"]}}]}}",
            owner_id,
            token_id
        ));
    }

    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>
    ) -> Vec<JsonToken> {
        if let Some(owner_tokens) = self.owners.get(&account_id) {
            let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
            let limit = limit.unwrap_or(50) as usize;
            
            owner_tokens.iter()
                .skip(start)
                .take(limit)
                .filter_map(|token_id| self.nft_token(format!("{}", token_id)))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        self.owners
            .get(&account_id)
            .map(|tokens| tokens.len() as u128)
            .unwrap_or(0)
            .into()
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
        assert_eq!(owner_tokens[0].token_id, token_id.to_string());

        // Transfer token
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .build());
        contract.nft_transfer(token_id.clone(), accounts(3));

        // Check new ownership
        let new_owner_tokens = contract.nft_tokens_for_owner(accounts(3), None, None);
        assert_eq!(new_owner_tokens.len(), 1);
        assert_eq!(new_owner_tokens[0].token_id, token_id.to_string());
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