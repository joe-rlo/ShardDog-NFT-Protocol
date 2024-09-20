const nearAPI = require('near-api-js');
const { keyStores } = nearAPI;
const { MerkleTree } = require('merkletreejs');
const crypto = require('crypto');
const db = require('./middle_db');
const SHA256 = require('sha256');



class NFTApi {
    constructor(config) {
        this.config = config;
        this.contractName = process.env.NEAR_CONTRACT_NAME;
    }


  
    async initialize() {
        console.log('Initializing NFTApi');
        if (!process.env.NEAR_ACCOUNT_ID || !process.env.NEAR_PRIVATE_KEY) {
            throw new Error('NEAR_ACCOUNT_ID or NEAR_PRIVATE_KEY is not set in environment variables');
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
            console.log('Connected to NEAR');
            
            this.account = await this.near.account(process.env.NEAR_ACCOUNT_ID);
            console.log('Account created');

            this.contract = new nearAPI.Contract(this.account, this.contractName, {
                viewMethods: ['get_channel_info', 'is_minted', 'get_next_token_number'],
                changeMethods: ['create_channel', 'nft_mint', 'update_merkle_root', 'nft_transfer', 'nft_burn'],
            });
            console.log('Contract instance created');
        } catch (error) {
            console.error('Error during initialization:', error);
            throw error;
        }
    }



    async createChannel(channelId, initialTokenIds, metadata) {
        console.log(`Creating channel with ID: ${channelId}`);
        console.log(`Initial token IDs: ${JSON.stringify(initialTokenIds)}`);
        
        // Ensure initialTokenIds is an array of strings
        const formattedInitialTokenIds = initialTokenIds.map(id => `${channelId}:${id}`);
        console.log(`Formatted initial token IDs: ${JSON.stringify(formattedInitialTokenIds)}`);
    
        const leaves = formattedInitialTokenIds.map(id => this.hashFunction(id));
        const merkleTree = new MerkleTree(leaves, this.hashFunction);
        const merkleRoot = merkleTree.getRoot();
    
        console.log(`Merkle root: ${merkleRoot.toString('hex')}`);
    
        await this.contract.create_channel(
            {
                channel_id: channelId,
                merkle_root: Array.from(merkleRoot), // Send as an array of numbers
                metadata: {
                    title: metadata.title,
                    description: metadata.description,
                    media: metadata.media,
                    animation_url: metadata.animation_url || null, // Use null if empty string
                    reference: metadata.reference,
                    reference_hash: metadata.reference_hash,
                },
            },
            300000000000000 // gas
        );
    
        // Store the merkle root as a hex string in the database
        const merkleRootHex = Buffer.from(merkleRoot).toString('hex');
        await db.createChannel(channelId, merkleRootHex, formattedInitialTokenIds);

        await db.updateChannelTokenCount(channelId, initialTokenIds.length);

        console.log(`Channel ${channelId} created with Merkle root: ${merkleRootHex} and ${initialTokenIds.length} initial tokens`);
    }

    async mintNextToken(channelId, accountid) {
        console.log(`Attempting to mint next token for channel: ${channelId}`);
    
        try {
            const nextTokenNumber = await this.contract.get_next_token_number({ channel_id: channelId });
            const tokenId = `${channelId}:${nextTokenNumber}`;
    
            let proof;
            let newRoot;
    
            if (nextTokenNumber === 1) {
                console.log(`Minting first token for channel ${channelId}`);
                proof = null; // No proof for the first token
                newRoot = this.hashFunction(tokenId);
            } else {
                // Get current token IDs and generate the new Merkle root
                const currentTokenIds = await db.getAllTokenIds(channelId);
                const allTokenIds = [...currentTokenIds, tokenId];
                const leaves = allTokenIds.map(id => this.hashFunction(id));
                const merkleTree = new MerkleTree(leaves, SHA256, { sortPairs: true });
                newRoot = merkleTree.getRoot();
    
                // Generate proof for the new token
                const leaf = this.hashFunction(tokenId);
                proof = merkleTree.getProof(leaf);
                console.log(`Generated Merkle proof: ${JSON.stringify(proof.map(item => item.data.toString('hex')))}`);
            }
    
            // Update the contract's Merkle root
            await this.contract.update_merkle_root(
                {
                    channel_id: channelId,
                    new_merkle_root: Array.from(newRoot),
                },
                300000000000000 // gas
            );
            console.log(`Updated contract Merkle root for channel ${channelId}: ${newRoot.toString('hex')}`);
    
            // Call the contract's mint method
            await this.contract.nft_mint(
                {
                    channel_id: channelId,
                    proof: proof ? proof.map(item => Array.from(item.data)) : null,
                    receiver_id: accountid,
                },
                300000000000000
            );
    
            // Update the database
            await db.mintToken(channelId, nextTokenNumber, this.account.accountId);
            await db.updateChannelTokenCount(channelId, nextTokenNumber);
            await db.updateChannelMerkleRoot(channelId, newRoot.toString('hex'));
    
            console.log(`Token minted successfully for channel ${channelId}, token number: ${nextTokenNumber}`);
            return tokenId;
        } catch (error) {
            console.error(`Error minting token:`, error);
            throw error;
        }
    }
    
    async generateMerkleProof(channelId, newTokenNumber) {
        console.log(`Generating Merkle proof for channel ${channelId}, new token number ${newTokenNumber}`);
        
        const currentTokenIds = await db.getAllTokenIds(channelId);
        console.log('Current Token IDs:', currentTokenIds);
        
        // Create a new array with the new token ID
        const allTokenIds = [...currentTokenIds, `${channelId}:${newTokenNumber}`];
        console.log('All Token IDs (including new):', allTokenIds);
        
        const leaves = allTokenIds.map(id => this.hashFunction(id));
        console.log('Leaves:', leaves.map(leaf => leaf.toString('hex')));
        
        const merkleTree = new MerkleTree(leaves, SHA256, { sortPairs: true });
        const root = merkleTree.getRoot();
        console.log('Merkle Root:', root.toString('hex'));
    
        const newTokenId = `${channelId}:${newTokenNumber}`;
        console.log('New Token ID:', newTokenId);
        
        const leaf = this.hashFunction(newTokenId);
        console.log('New Leaf:', leaf.toString('hex'));
        
        const proof = merkleTree.getProof(leaf);
        console.log('Raw Proof:', proof.map(item => item.data.toString('hex')));
    
        // Verify the proof
        const isValid = merkleTree.verify(proof, leaf, root);
        console.log(`Proof verification result: ${isValid}`);
    
        if (!isValid) {
            throw new Error('Merkle proof verification failed');
        }
    
        return proof.map(item => Array.from(item.data));
    }
    
    async updateMerkleTreeForNewToken(channelId, newTokenNumber) {
        const currentTokenIds = await db.getAllTokenIds(channelId);
        const newTokenId = `${channelId}:${newTokenNumber}`;
        currentTokenIds.push(newTokenId);
        
        console.log('Updated Token IDs:', currentTokenIds);
        
        const leaves = currentTokenIds.map(id => this.hashFunction(id));
        const merkleTree = new MerkleTree(leaves, SHA256, { sortPairs: true });
        const newRoot = merkleTree.getRoot();
        
        await db.updateChannelMerkleRoot(channelId, newRoot.toString('hex'));
        
        await this.contract.update_merkle_root(
            {
                channel_id: channelId,
                new_merkle_root: Array.from(newRoot),
            },
            300000000000000 // gas
        );
        
        console.log(`Updated Merkle root for channel ${channelId}: ${newRoot.toString('hex')}`);
    }
    
    hashFunction(data) {
        return crypto.createHash('sha256').update(Buffer.from(data)).digest();
    }
    
    verifyMerkleProof(root, tokenId, proof) {
        let computedHash = this.hashFunction(tokenId);
        console.log(`Initial hash: ${computedHash.toString('hex')}`);
        console.log(`Token ID: ${tokenId}`);
    
        for (let i = 0; i < proof.length; i++) {
            const proofElement = Buffer.from(proof[i]);
            console.log(`Proof element ${i}: ${proofElement.toString('hex')}`);
            let combined;
            if (Buffer.compare(computedHash, proofElement) <= 0) {
                combined = Buffer.concat([computedHash, proofElement]);
            } else {
                combined = Buffer.concat([proofElement, computedHash]);
            }
            computedHash = this.hashFunction(combined);
            console.log(`After step ${i}: ${computedHash.toString('hex')}`);
        }
    
        console.log(`Final hash: ${computedHash.toString('hex')}`);
        console.log(`Root: ${Buffer.from(root).toString('hex')}`);
    
        return Buffer.compare(computedHash, Buffer.from(root)) === 0;
    }


    

    updateMerkleTree(channelId, newTokenNumber) {
        const merkleTree = this.merkleTrees.get(channelId);
        if (!merkleTree) {
            throw new Error(`Merkle tree not found for channel: ${channelId}`);
        }

        const newLeaf = this.hashFunction(`${channelId}:${newTokenNumber}`);
        merkleTree.addLeaf(newLeaf);
    }

    async transferToken(channelId, tokenNumber, receiverId) {
        const currentOwner = await db.getTokenOwner(channelId, tokenNumber);

        if (currentOwner !== this.account.accountId) {
            throw new Error("Only the token owner can transfer the token");
        }

        const tokenId = `${channelId}:${tokenNumber}`;

        await this.contract.nft_transfer(
            {
                token_id: tokenId,
                receiver_id: receiverId,
            },
            300000000000000, // gas
            1 // attached deposit (1 yoctoNEAR for security reasons)
        );

        await db.transferToken(channelId, tokenNumber, receiverId);

        console.log(`Token ${tokenId} transferred to ${receiverId}`);
    }

    async burnToken(channelId, tokenNumber) {
        const currentOwner = await db.getTokenOwner(channelId, tokenNumber);

        if (currentOwner !== this.account.accountId) {
            throw new Error("Only the token owner can burn the token");
        }

        const tokenId = `${channelId}:${tokenNumber}`;

        await this.contract.nft_burn(
            {
                token_id: tokenId,
            },
            300000000000000, // gas
            1 // attached deposit (1 yoctoNEAR for security reasons)
        );

        await db.burnToken(channelId, tokenNumber);

        const channelInfo = await db.getChannelInfo(channelId);
        const newMerkleRoot = this.calculateNewMerkleRoot(channelInfo, tokenNumber);
        await db.updateChannelMerkleRoot(channelId, newMerkleRoot);

        await this.contract.update_merkle_root(
            {
                channel_id: channelId,
                new_merkle_root: Array.from(Buffer.from(newMerkleRoot, 'hex')),
            },
            300000000000000 // gas
        );

        console.log(`Token ${tokenId} burned and Merkle root updated`);
    }

    calculateNewMerkleRoot(channelInfo, burnedTokenNumber) {
        const currentTokenIds = channelInfo.tokenIds.filter(id => !id.endsWith(`:${burnedTokenNumber}`));
        const leaves = currentTokenIds.map(id => this.hashFunction(id));
        const merkleTree = new MerkleTree(leaves, SHA256, { sortPairs: true });
        return merkleTree.getRoot().toString('hex');
    }
    
}

module.exports = NFTApi;