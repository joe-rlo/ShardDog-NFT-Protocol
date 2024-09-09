# ShardNFTs

This NFT contract implements a flexible and efficient system for managing multiple NFT collections (called "channels") using Merkle trees. Here's a high-level overview:

The contract allows creation of multiple channels, each with its own metadata and Merkle root. Tokens are minted by providing a Merkle proof, which is verified against the channel's root. The contract stores minimal on-chain data: channel information, minted token IDs, and contract metadata. The off-chain component (middleware) manages the Merkle trees, token ownership, and generates proofs for minting. This design allows for efficient scaling of large NFT collections while minimizing on-chain storage costs. The middleware handles complex operations like creating channels, minting tokens, and updating Merkle roots, while the contract verifies proofs and maintains the core state. This hybrid approach balances the benefits of blockchain security with off-chain computational efficiency.

# 1. Deploy the Smart Contract

Build the contract
cargo build --target wasm32-unknown-unknown --release

Deploy the contract
near deploy --accountId your-contract.testnet --wasmFile target/wasm32-unknown-unknown/release/nft_contract.wasm

Initialize the contract
near call your-contract.testnet new '{"owner_id": "your-account.testnet"}' --accountId your-account.testnet

2. Set up ClickHouse Database

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


3. Set up and Run the Off-Chain Application
See deploy.md

4. Interacting with the System

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

5. Querying Data (ClickHouse)

Get collection info
clickhouse-client --query "SELECT * FROM nft_db.channels WHERE channel_id = 'channel1'"

Get tokens owned by an account
clickhouse-client --query "SELECT * FROM nft_db.owner_tokens_mv WHERE owner_id = 'your-account.testnet'"

Get all tokens in a collection
clickhouse-client --query "SELECT * FROM nft_db.tokens WHERE channel_id = 'channel1'"

Get total supply of a collection
clickhouse-client --query "SELECT total_supply FROM nft_db.channels WHERE channel_id = 'channel1'"

--- 

How the Contract Works:
Channel Creation:
- The contract owner can create channels, each with its own Merkle root.
- Each channel keeps track of its total supply and next token number.

Minting Process:
- Users can mint tokens by providing a channel ID and a Merkle proof.
- The contract verifies the Merkle proof against the stored root for the channel.
- If valid, it mints the token with the next available token number for that channel.
- Token IDs are formatted as "{channel_id}:{token_number}".

Merkle Tree Usage:
- The Merkle tree root is stored on-chain for each channel.
- Proofs are generated off-chain and provided during minting.

How to Use the Contract:
Deploy the Contract:
- Deploy the contract to the NEAR network, specifying the owner account.

Create a Channel:
```
await contract.create_channel({
     channel_id: "my_channel",
     merkle_root: [/* array of bytes representing the Merkle root */]
   });
```

Prepare for Minting:
- Off-chain, generate a Merkle tree for the tokens you want to allow minting.
- Store the token IDs and tree structure off-chain.

Mint a Token:
```
await contract.mint({
     channel_id: "my_channel",
     proof: [/* array of proof elements */]
   }, gas, deposit);
```

Update Merkle Root (if needed):
```
await contract.update_merkle_root({
     channel_id: "my_channel",
     new_merkle_root: [/* array of bytes representing the new Merkle root */]
   });
```

View Functions:
- Check if a token is minted: await contract.is_minted({ token_id: "my_channel:1" });
- Get channel info: await contract.get_channel_info({ channel_id: "my_channel" });
- Get next token number: await contract.get_next_token_number({ channel_id: "my_channel" });