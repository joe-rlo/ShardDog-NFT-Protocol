# Changelog

All notable changes to the ShardDog NFT Protocol will be documented in this file.

## [0.3.0] - 2025-01-01

### Changed
- Modified channel creation to support unlimited minting
  - Set `total_possible` to `u64::MAX` by default
  - Removed `total_possible` parameter from channel creation interface
  - Removed minting limit checks from `nft_mint` function

### Fixed
- Removed unnecessary manual refund logic in `nft_mint` function
  - NEAR protocol handles excess deposit refunds automatically
  - Simplified storage cost handling
- Fixed JSON serialization issues in minting process
  - Updated middleware to properly format parameters
  - Improved error handling for contract interactions

### Technical Details
- Storage cost calculation remains unchanged
- Merkle proof verification still required for non-first tokens
- NEP-171 event emission maintained
- Database schema and operations unchanged

### Migration Notes
- Existing channels with limited minting should be recreated to enable unlimited minting
- No database migrations required
- No changes needed to frontend integration

### Testing
- Verified minting functionality with new unlimited channels
- Confirmed storage cost calculations working correctly
- Validated NEP-171 event emissions

## [0.2.0] - 2024-10-23

### Changed
- Switched from individual token storage to owner-centric token tracking
- Burn tokens with ownership checks

## [0.1.0] - 2024-9-24

### Added
- Initial release of the ShardDog NFT Protocol
- Multiple channel support within a single contract
- Merkle tree verification for efficient token minting
- Owner-centric token tracking system
- Create channel functionality
- Mint tokens with Merkle proof verification
- Transfer tokens between account
- View functions for querying token ownership and channel info
- Integration with ClickHouse database for off-chain data management
- Implemented Merkle tree approach for token verification instead of on-chain metadata storage



