# NEAR NFT Collection Middleware

This middleware facilitates the creation and management of NFT collections on the NEAR Protocol, utilizing a Merkle tree approach for efficient storage and minting. It's designed to work with Next.js and NEAR API JS.

## Key Features

1. **Efficient Storage**: Uses a Merkle tree approach to minimize on-chain storage costs.
2. **Channel-based Collections**: Organizes NFTs into channels (series) for better management.
3. **Owner-Centric Token Tracking**: Efficiently tracks token ownership without storing individual token data.
4. **Minting**: Allows minting of new tokens within a channel.
5. **Transfers**: Supports token transfers between accounts.
6. **Burns**: Allows token burning with appropriate ownership checks.
7. **Ownership Queries**: Provides functions to lookup owned tokens and token supply per owner.

## How It Works

1. **Channel Creation**: 
   - Create a new channel (series) with a Merkle root and metadata.
   - Each channel has its own Merkle tree for token verification.

2. **Minting**:
   - Mint new tokens within a channel.
   - Tokens are added to the owner's set of owned tokens.
   - The contract maintains a global set of minted tokens.

3. **Ownership Tracking**:
   - Uses a LookupMap to associate accounts with their owned tokens.
   - Efficient for users owning multiple tokens.

4. **Transfers**:
   - Allows transferring tokens between accounts.
   - Updates the ownership records accordingly.

5. **Burns**:
   - Removes tokens from circulation.
   - Updates channel supply and ownership records.

6. **Queries**:
   - Supports querying tokens owned by an account.
   - Provides token supply information per owner and channel.

## Integration

This middleware is designed to work with Next.js for the frontend and NEAR API JS for blockchain interactions. It provides a set of contract functions that can be called from your application to manage NFT collections efficiently.

## Getting Started

[Include instructions on how to set up and use the middleware in a Next.js project, including any necessary configuration for NEAR API JS.]

## API Reference

[Include a brief overview of the main contract functions, their parameters, and return values.]

## Notes

- This implementation prioritizes storage efficiency while maintaining functionality.
- It does not store individual token metadata on-chain, instead relying on the Merkle tree approach for verification.
- Suitable for large-scale NFT collections where minimizing storage costs is crucial.
