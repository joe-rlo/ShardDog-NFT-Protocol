const nearAPI = require('near-api-js');
const { utils } = nearAPI;
const { MerkleTree } = require('merkletreejs');
const crypto = require('crypto');

class TokenManager {
    constructor(channelId, initialTokenNumbers = []) {
        this.channelId = channelId;
        this.tokenNumbers = new Set(initialTokenNumbers);
        this.nextTokenNumber = Math.max(...initialTokenNumbers, 0) + 1;
        this.tokenOwners = new Map();
        this.updateMerkleTree();
    }

    updateMerkleTree() {
        const leaves = Array.from(this.tokenNumbers).map(number => 
            crypto.createHash('sha256').update(`${this.channelId}:${number}`).digest()
        );
        this.merkleTree = new MerkleTree(leaves, crypto.createHash('sha256'));
    }

    addNextToken() {
        this.tokenNumbers.add(this.nextTokenNumber);
        this.nextTokenNumber++;
        this.updateMerkleTree();
    }

    getMerkleProof(tokenNumber) {
        const tokenId = `${this.channelId}:${tokenNumber}`;
        const leaf = crypto.createHash('sha256').update(tokenId).digest();
        return this.merkleTree.getProof(leaf).map(x => Array.from(x.data));
    }

    getMerkleRoot() {
        return this.merkleTree.getRoot();
    }

    removeToken(tokenNumber) {
        this.tokenNumbers.delete(tokenNumber);
        this.updateMerkleTree();
    }

    setTokenOwner(tokenNumber, owner) {
        this.tokenOwners.set(tokenNumber, owner);
    }

    getTokenOwner(tokenNumber) {
        return this.tokenOwners.get(tokenNumber);
    }

    transferToken(tokenNumber, newOwner) {
        if (this.tokenOwners.has(tokenNumber)) {
            this.tokenOwners.set(tokenNumber, newOwner);
            return true;
        }
        return false;
    }
}

// Set up your NEAR connection and contract
const near = await nearAPI.connect(config);
const account = await near.account(accountId);
const contract = new nearAPI.Contract(account, contractName, {
    viewMethods: ['get_channel_info', 'is_minted'],
    changeMethods: ['create_channel', 'mint', 'update_merkle_root', 'transfer', 'burn'],
});

// Create a channel
async function createChannel(channelId, initialTokenIds, metadata) {
    const tokenManager = new TokenManager(channelId, initialTokenIds);
    const merkleRoot = tokenManager.getMerkleRoot();

    await contract.create_channel(
        {
            channel_id: channelId,
            merkle_root: Array.from(merkleRoot),
            metadata: {
                title: metadata.title,
                description: metadata.description,
                media: metadata.media,
                animation_url: metadata.animation_url,
                reference: metadata.reference,
                reference_hash: metadata.reference_hash,
            },
        },
        300000000000000, // gas
        utils.format.parseNearAmount("0.1") // deposit (if required)
    );

    console.log(`Channel ${channelId} created with Merkle root: ${merkleRoot.toString('hex')}`);
    return tokenManager;
}

// Append tokens to an existing channel
async function appendTokens(channelId, tokenManager, newTokenIds) {
    tokenManager.addTokens(newTokenIds);
    const newMerkleRoot = tokenManager.getMerkleRoot();

    await contract.update_merkle_root({
        channel_id: channelId,
        new_merkle_root: Array.from(newMerkleRoot),
    });

    console.log(`Channel ${channelId} updated with new Merkle root: ${newMerkleRoot.toString('hex')}`);
}

// Function to get the next token number from the contract
async function getNextTokenNumber(channelId) {
    return await contract.get_next_token_number({ channel_id: channelId });
}

// Function to mint the next available token
async function mintNextToken(channelId, tokenManager) {
    const nextTokenNumber = await getNextTokenNumber(channelId);
    
    // Ensure the off-chain TokenManager is in sync with the contract
    while (tokenManager.nextTokenNumber <= nextTokenNumber) {
        tokenManager.addNextToken();
    }

    const proof = tokenManager.getMerkleProof(nextTokenNumber);

    await contract.mint(
        {
            channel_id: channelId,
            proof: proof,
        },
        300000000000000, // gas
        utils.format.parseNearAmount("0.1") // deposit (if required)
    );

    tokenManager.setTokenOwner(nextTokenNumber, account.accountId);

    console.log(`Token ${channelId}:${nextTokenNumber} minted and owned by ${account.accountId}`);
}

async function transferToken(channelId, tokenNumber, receiverId, tokenManager) {
    const tokenId = `${channelId}:${tokenNumber}`;
    const currentOwner = tokenManager.getTokenOwner(tokenNumber);

    if (currentOwner !== account.accountId) {
        throw new Error("Only the token owner can transfer the token");
    }

    await contract.transfer(
        {
            token_id: tokenId,
            receiver_id: receiverId,
        },
        300000000000000, // gas
        1 // attached deposit (1 yoctoNEAR for security reasons)
    );

    // Update ownership in TokenManager
    tokenManager.transferToken(tokenNumber, receiverId);

    console.log(`Token ${tokenId} transferred to ${receiverId}`);
}

async function burnToken(channelId, tokenNumber, tokenManager) {
    const tokenId = `${channelId}:${tokenNumber}`;
    const currentOwner = tokenManager.getTokenOwner(tokenNumber);

    if (currentOwner !== account.accountId) {
        throw new Error("Only the token owner can burn the token");
    }

    await contract.burn(
        {
            token_id: tokenId,
        },
        300000000000000, // gas
        1 // attached deposit (1 yoctoNEAR for security reasons)
    );

    // Update the local TokenManager
    tokenManager.removeToken(tokenNumber);
    tokenManager.tokenOwners.delete(tokenNumber);

    // Update the Merkle root on the contract
    const newMerkleRoot = tokenManager.getMerkleRoot();
    await contract.update_merkle_root(
        {
            channel_id: channelId,
            new_merkle_root: Array.from(newMerkleRoot),
        },
        300000000000000 // gas
    );

    console.log(`Token ${tokenId} burned and Merkle root updated`);
}


async function main() {
    const channelId = "my_channel";
    const metadata = {
        title: "My Channel",
        description: "This is my channel",
        media: "https://example.com/media.mp4",
        animation_url: null,
        reference: null,
        reference_hash: null, // Add a Base64-encoded string here if needed
    };

    // Create a new channel with metadata
    let tokenManager = await createChannel(channelId, [1, 2, 3, 4, 5], metadata);

    // Mint the next available token
    await mintNextToken(channelId, tokenManager);

        // Transfer the minted token
        const tokenId = `${channelId}:6`; // Assuming the minted token is number 6
        const receiverId = "receiver.testnet";
        await transferToken(channelId, tokenNumber, receiverId, tokenManager);
    
     // Burn a token and update Merkle root
     const tokenToBurn = 1; // Burning token number 1
     await burnToken(channelId, tokenToBurn, tokenManager);
}

main().catch(console.error);