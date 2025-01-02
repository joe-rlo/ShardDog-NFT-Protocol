const nearAPI = require('near-api-js');
const { keyStores } = nearAPI;
const db = require('./middle_db');
const crypto = require('crypto');

class NFTApi {
    constructor(config) {
        this.config = config;
        this.contractName = process.env.NEAR_CONTRACT_NAME;
    }

    async initialize() {
        console.log('Initializing NFTApi');
        if (!process.env.NEAR_ACCOUNT_ID || !process.env.NEAR_PRIVATE_KEY) {
            throw new Error('NEAR_ACCOUNT_ID or NEAR_PRIVATE_KEY is not set');
        }

        const keyPair = nearAPI.utils.KeyPair.fromString(process.env.NEAR_PRIVATE_KEY);
        const keyStore = new keyStores.InMemoryKeyStore();
        await keyStore.setKey(this.config.networkId, process.env.NEAR_ACCOUNT_ID, keyPair);

        const nearConfig = {
            ...this.config,
            keyStore,
            headers: {}
        };

        try {
            this.near = await nearAPI.connect(nearConfig);
            this.account = await this.near.account(process.env.NEAR_ACCOUNT_ID);
            this.contract = new nearAPI.Contract(this.account, this.contractName, {
                viewMethods: ['nft_metadata', 'get_channel_info', 'is_minted', 'get_next_token_number', 'nft_token', 'nft_tokens_for_owner', 'nft_supply_for_owner'],
                changeMethods: ['create_channel', 'update_channel', 'nft_mint', 'nft_transfer', 'nft_burn'],
            });
            console.log('Contract instance created');
        } catch (error) {
            console.error('Error during initialization:', error);
            throw error;
        }
    }

    async createChannel(channelId, merkleRoot, metadata) {
        console.log(`Creating channel with ID: ${channelId}`);
        
        try {
            // Call contract to create channel
            await this.contract.create_channel(
                {
                    channel_id: channelId,
                    merkle_root: Array.from(merkleRoot),
                    metadata: {
                        title_template: metadata.title_template,
                        description_template: metadata.description_template,
                        media: metadata.media,
                        animation_url: metadata.animation_url || null,
                        reference: metadata.reference,
                        reference_hash: metadata.reference_hash,
                    },
                },
                300000000000000 // gas
            );

            // Store in database
            await db.createChannel(channelId, Buffer.from(merkleRoot).toString('hex'), []);
            
            console.log(`Channel ${channelId} created successfully`);
        } catch (error) {
            console.error(`Error creating channel:`, error);
            throw error;
        }
    }

    async updateChannel(channelId, merkleRoot, metadata) {
        console.log(`Updating channel with ID: ${channelId}`);
        
        try {
            await this.contract.update_channel(
                {
                    channel_id: channelId,
                    merkle_root: merkleRoot ? Array.from(merkleRoot) : null,
                    metadata: metadata ? {
                        title_template: metadata.title_template,
                        description_template: metadata.description_template,
                        media: metadata.media,
                        animation_url: metadata.animation_url || null,
                        reference: metadata.reference,
                        reference_hash: metadata.reference_hash,
                    } : null,
                },
                300000000000000 // gas
            );

            if (merkleRoot) {
                await db.updateChannelMerkleRoot(channelId, Buffer.from(merkleRoot).toString('hex'));
            }
            
            console.log(`Channel ${channelId} updated successfully`);
        } catch (error) {
            console.error(`Error updating channel:`, error);
            throw error;
        }
    }

    async mintNFT(channelId, proof, receiverId) {
        console.log(`Minting NFT for channel: ${channelId}`);
        
        try {
            // Calculate required storage deposit
            const storageCost = await this.calculateStorageCost();
            
            // Call contract to mint NFT
            const tokenId = await this.contract.nft_mint(
                {
                    channel_id: channelId,
                    proof: proof ? proof.map(p => Array.from(p)) : null,
                    receiver_id: receiverId,
                },
                300000000000000, // gas
                storageCost // attached deposit for storage
            );

            // Update database
            const [, tokenNumber] = tokenId.split(':');
            await db.mintToken(channelId, parseInt(tokenNumber), receiverId);
            
            console.log(`NFT minted successfully: ${tokenId}`);
            return tokenId;
        } catch (error) {
            console.error(`Error minting NFT:`, error);
            throw error;
        }
    }

    async calculateStorageCost() {
        // Simulate storage operations to calculate cost
        const initialStorage = await this.account.getAccountBalance();
        // Add buffer for safety
        return BigInt(initialStorage.storage_usage) * BigInt(10); // Multiply by 10 for safety margin
    }

    async transferNFT(tokenId, receiverId) {
        console.log(`Transferring NFT ${tokenId} to ${receiverId}`);
        
        try {
            await this.contract.nft_transfer(
                {
                    receiver_id: receiverId,
                    token_id: tokenId,
                    approval_id: null,
                    memo: null,
                },
                300000000000000, // gas
                1 // one yoctoNEAR for security
            );

            const [channelId, tokenNumber] = tokenId.split(':');
            await db.transferToken(channelId, parseInt(tokenNumber), receiverId);
            
            console.log(`NFT transferred successfully`);
        } catch (error) {
            console.error(`Error transferring NFT:`, error);
            throw error;
        }
    }

    async burnNFT(tokenId) {
        console.log(`Burning NFT ${tokenId}`);
        
        try {
            await this.contract.nft_burn(
                {
                    token_id: tokenId,
                },
                300000000000000, // gas
                1 // one yoctoNEAR for security
            );

            const [channelId, tokenNumber] = tokenId.split(':');
            await db.burnToken(channelId, parseInt(tokenNumber));
            
            console.log(`NFT burned successfully`);
        } catch (error) {
            console.error(`Error burning NFT:`, error);
            throw error;
        }
    }
}

module.exports = NFTApi;