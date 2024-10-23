# ShardDog NFT Protocol

[See Blog Post HERE](https://shard.dog/sharddog-protocol)

This NFT contract implements a flexible and efficient system for managing multiple NFT collections (called "channels") using Merkle trees and owner-centric token tracking. 

The contract allows creation of multiple channels, each with its own metadata and Merkle root. Tokens are minted by providing a Merkle proof, which is verified against the channel's root. The contract stores minimal on-chain data: channel information, minted token IDs, and contract metadata. The off-chain component (middleware) manages the Merkle trees, token ownership, and generates proofs for minting. This design allows for efficient scaling of large NFT collections while minimizing on-chain storage costs. The middleware handles complex operations like creating channels, minting tokens, and updating Merkle roots, while the contract verifies proofs and maintains the core state. This hybrid approach balances the benefits of blockchain security with off-chain computational efficiency.

Here's a high-level overview:

## Key Features

1. **Multiple Channels**: Create and manage multiple NFT collections within a single contract.
2. **Merkle Tree Verification**: Use Merkle trees for efficient token verification during minting.
3. **Owner-Centric Token Tracking**: Efficiently track token ownership without storing individual token data.
4. **Minimal On-Chain Storage**: Store only essential data on-chain to minimize costs.
5. **Flexible Minting**: Allow minting of new tokens within a channel with Merkle proof verification.
6. **Token Transfers**: Support token transfers between accounts.
7. **Token Burns**: Allow token burning with appropriate ownership checks.
8. **Ownership Queries**: Provide functions to lookup owned tokens and token supply per owner.

## How It Works

1. **Channel Creation**: 
   - The contract owner can create channels, each with its own metadata and Merkle root.
   - Each channel keeps track of its total supply and next token number.

2. **Minting Process**:
   - Users can mint tokens by providing a channel ID and a Merkle proof.
   - The contract verifies the Merkle proof against the stored root for the channel.
   - If valid, it mints the token and assigns ownership to the minter.
   - Token IDs are formatted as "{channel_id}:{token_number}".

3. **Ownership Tracking**:
   - The contract uses a LookupMap to associate accounts with their owned tokens.
   - This approach is efficient for users owning multiple tokens.

4. **Transfers and Burns**:
   - Users can transfer their tokens to other accounts.
   - Token burning is supported, updating channel supply and ownership records.

5. **Merkle Tree Usage**:
   - Merkle tree roots are stored on-chain for each channel.
   - Proofs are generated off-chain and provided during minting.

## How to Use the Contract

1. **Deploy the Contract**:
   ```bash
   near deploy --accountId your-contract.testnet --wasmFile target/wasm32-unknown-unknown/release/nft_contract.wasm
   near call your-contract.testnet new '{"owner_id": "your-account.testnet"}' --accountId your-account.testnet
   ```

2. **Create a Channel**:
   ```javascript
   await contract.create_channel({
     channel_id: "my_channel",
     merkle_root: [/* array of bytes representing the Merkle root */],
     metadata: {
       title: "My Channel",
       description: "Description of my channel",
       media: "https://example.com/image.jpg",
       // ... other metadata fields
     }
   });
   ```

3. **Mint a Token**:
   ```javascript
   await contract.mint({
     channel_id: "my_channel",
     proof: [/* array of proof elements */]
   }, gas, deposit);
   ```

4. **Transfer a Token**:
   ```javascript
   await contract.nft_transfer({
     token_id: "my_channel:1",
     receiver_id: "receiver.testnet"
   });
   ```

5. **Burn a Token**:
   ```javascript
   await contract.nft_burn({
     token_id: "my_channel:1"
   });
   ```

6. **View Functions**:
   ```javascript
   // Get tokens owned by an account
   await contract.nft_tokens_for_owner({ account_id: "owner.testnet" });
   
   // Get channel info
   await contract.get_channel_info({ channel_id: "my_channel" });
   
   // Get token info
   await contract.nft_token({ token_id: "my_channel:1" });
   ```

## Deploying the Contract

Build the contract
cargo build --target wasm32-unknown-unknown --release

Deploy the contract
near deploy --accountId your-contract.testnet --wasmFile target/wasm32-unknown-unknown/release/nft_contract.wasm

Initialize the contract
near call your-contract.testnet new '{"owner_id": "your-account.testnet"}' --accountId your-account.testnet

## Set up ClickHouse Database

Install ClickHouse (Ubuntu example)
sudo apt-get install clickhouse-server clickhouse-client

Start ClickHouse server
sudo service clickhouse-server start

Create the database schema
clickhouse-client --multiline 
CREATE DATABASE IF NOT EXISTS nft_db;

USE nft_db;

-- Create tables as defined in the previous "clickhouse-schema" artifact
-- (collections, tokens, owner_tokens_mv, merkle_nodes)


-- Create the channels table (previously collections)
CREATE TABLE channels
(
    channel_id String,
    merkle_root String,
    total_supply UInt64,
    next_token_number UInt64,
    created_at DateTime DEFAULT now(),
    updated_at DateTime DEFAULT now()
)
ENGINE = MergeTree()
ORDER BY (channel_id);

-- Create the tokens table
CREATE TABLE tokens
(
    token_id String,
    channel_id String,
    owner_id String,
    token_number UInt64,
    minted_at DateTime DEFAULT now()
)
ENGINE = MergeTree()
ORDER BY (channel_id, token_number);

-- Create a materialized view for quick owner queries
CREATE MATERIALIZED VIEW owner_tokens_mv
ENGINE = SummingMergeTree()
ORDER BY (owner_id, channel_id)
POPULATE
AS SELECT
    owner_id,
    channel_id,
    count() AS token_count
FROM tokens
GROUP BY owner_id, channel_id;

-- Create a table for storing Merkle tree nodes (optional, for full tree storage)
CREATE TABLE merkle_nodes
(
    collection_id String,
    node_hash String,
    parent_hash String,
    level UInt8,
    created_at DateTime DEFAULT now()
)
ENGINE = MergeTree()
ORDER BY (collection_id, level, node_hash);


## Set up and Run the Off-Chain Application
See deploy.md

## Interacting with the Contract

Create a new channel
near call your-contract.testnet create_channel '{"channel_id": "channel1", "merkle_root": []}' --accountId your-account.testnet

Mint a new token (this involves both off-chain and on-chain operations)
First, use the off-chain application to prepare the minting data
Then, call the smart contract to mint the token
near call your-contract.testnet mint '{
  "channel_id": "channel1", 
  "proof": [[0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1]]
}' --accountId your-account.testnet

Update Merkle root (after off-chain changes)
near call your-contract.testnet update_merkle_root '{
  "channel_id": "channel1", 
  "new_merkle_root": [0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1]
}' --accountId your-account.testnet

## Querying Data (ClickHouse)

Get collection info
clickhouse-client --query "SELECT * FROM nft_db.channels WHERE channel_id = 'channel1'"

Get tokens owned by an account
clickhouse-client --query "SELECT * FROM nft_db.owner_tokens_mv WHERE owner_id = 'your-account.testnet'"

Get all tokens in a collection
clickhouse-client --query "SELECT * FROM nft_db.tokens WHERE channel_id = 'channel1'"

Get total supply of a collection
clickhouse-client --query "SELECT total_supply FROM nft_db.channels WHERE channel_id = 'channel1'"

## Notes

- This implementation prioritizes storage efficiency while maintaining functionality.
- It does not store individual token metadata on-chain, instead relying on the Merkle tree approach for verification.
- Suitable for large-scale NFT collections where minimizing storage costs is crucial.
- The off-chain component (middleware) manages the Merkle trees, token metadata, and generates proofs for minting.

For more detailed information on setting up the off-chain components and interacting with the ClickHouse database, please refer to the deployment guide and additional documentation.
