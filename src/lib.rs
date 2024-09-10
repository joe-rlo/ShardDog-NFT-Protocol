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
        let contract = Self {
            owner_id: owner_id.clone(),
            channels: UnorderedMap::new(b"c"),
            minted_tokens: UnorderedSet::new(b"m"),
            metadata: NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "ShardDog".to_string(),
                symbol: "SHARDDOG".to_string(),
                icon: Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAIAAAACACAMAAAD04JH5AAADAFBMVEUAAAD////17eAAXMsAK8QAL8YANscAMcIAOsgAOMYAOcQANsMAOcMAPcgAPccAP8cAPsUAO8UAPcUAPsMAO8AAQcoARMoAQcgAQsgAQscAQcYAQsUAQcUAQcIAScsAR8sASMkARsgARcgASccARscARccAR8UARMUAR8MCSccATcwAT8sAS8oATMoAS8kATMgATccASsYASsQAScMET8YLT8QNVMYxbMk8c8phissAUswAUMsAUssAUMcAUcYATcQFUscFUcMHVsYYXccjZ8g5csdShMxdiMtlkM52ms2IpdG4xNYAVswAVcwAWcwAVcsAVssAV8gAVMgAU8cAVccDWMcEWcgMW8oWYMgZZsoobspIgMxOg86gt9WpvdfCzNkAWswAXMwAWcsAW8sAXcsAWMgAWsgDXswHX8oMYMkOY8oOX8YTZcgba8oxdco1ess/gM5Cgc9Egs9DgctDf8lIh9JHhM5GgslKhs1Nhc1NhctTis5XjtBZjc5ckdBekM9hks9gkMxllc5qmNJplcxumtFzntF7otKDp9OApM+MrNSSr9KbtNOsv9bHz9kAYMwAX8wAXssSaMojccssdswveMw7fstIhstPicxmmNB3odJ7ptV7os+Iq9KUstSxxdtEhcqxxNe+y9iKsNKcudSjvNPFztbM1NutxNXS1tm5ytXQ19u9ztfAztXW2tyyy9fT2tzP29rQ3dvW2tnV3dnb4drk6d7q8N/g4dv//+P//+X//+b//+jx8d7c3Nr//uH//eH//eP//eT9+uL2897/++Ts6t3/+uD9+N//+uP79t/8+OPi4dz/+d/n5d3/+eP/+OHo5dv/9+D99uP89eL79eP58+L/9+P/9+T78+L38ODx693q5t3/9eD88t317uDa2df/9eL98+H88+L78uH58OD48OLm4tv68OD47+D17eDw6d3/8+H88eH57t737uD27eD17N/07OD+8uL169368OL069/z6+D67+Hz6t/06+Dm4t327OHk393a2Nj///+GvQ8uAAABAHRSTlP///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////8AU/cHJQAAAAlwSFlzAAALEwAACxMBAJqcGAAACW1JREFUeJztmnlwE9cdx7+7kmxZl+VDtuXYgA8wPjkdc6Rgc5YMNDMdoDOdhlgpoZ02YA+kzfAHnTLTEspgc2QyQykZCKTTGcq0U9LWZAL4mEAA2xhsE3zJxviSLR9ro8uSVts/ZJHG2rWeLAH/6P1j777fe9+Pfu/33v7e26VovNryqvVDACGAEEAIIAQQAoA4+F3qHXIR1OOWNCJrKpguGDX1MaA4hNEcZwfFzdHS6pcGwKjrh13x2btd2Q670wqpRCqpPtmVnBrxkgCshpbUq7Yhe7PBzEWAAwWzKD4jd4cj10fDIMUA0zHRc6x3UJ20YMGF5zc31V+hlo9GvwwARtRf+7sFK8Xne775v7uXdIORHOujaVCGgDE1jR9RLDzvVfHm4OX2tdIZ2wYlBNhm4wHHdm99rG/9r3h05rZB8UCVsVS55wBfzZaJqq8LZ2wbDH27rXmYXx/xhmLbzI2DAdCijEnj1wc4jnvhALaB20fzBepYxSilB6AXbB2EaWgQS+0igbr6PTl2ZX84JbwYBB6EtuqJfRsvCVSuTN6ybrLDo5WhUXhZBO6BLuUXjiVCANuvlNVmdCqk4uZc0JqCJ4x6ukXgHrjWcPgnFwVrda67IpOFo2W0STZYsnF+8nSDgD3Q36KUC0UAgKnVqZgVga0cm+P0MggYYHhORYKPmQbgAgDMVfVkeNUEPAJ5HRl2QlNJrl0WfIAerslxgcRQV+AUu9TBBxBDNUMIfFfK7j4+kJnlfT9gAC0tN5Pon8tetZSOfwEA4Ghql2/9I3H7SiJTeVKDwNeBRubBmceCwlWNADjrZPxHinzeFDnwlTC9pntEqO74uaidcMGprP2VJJ//eRCEh5GEEkq9yz6V7k5AGKjJN8ILGX6bgAFMRjZKwAO6y9bfq3IpJyDWAuoXA8A8GR9s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".to_string()),
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


    #[payable]
    pub fn mint(&mut self, channel_id: String, proof: Option<Vec<Vec<u8>>>) -> Promise {
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
    
        let owner_id = env::predecessor_account_id();
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
        "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_mint\",\"data\":[{{\"owner_id\":\"{}\",\"token_ids\":[\"{}\"],\"memo\":null}}]}}",
        owner_id, token_id
    ));

    // Log additional metadata in a separate event
    env::log_str(&format!(
        "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_metadata\",\"data\":{{\"token_id\":\"{}\",\"metadata\":{}}}}}",
        token_id, token_metadata.to_string()
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