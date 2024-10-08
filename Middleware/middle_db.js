const { createClient } = require('@clickhouse/client');

const client = createClient({
  host: process.env.CLICKHOUSE_HOST,
  database: process.env.CLICKHOUSE_DATABASE,
  username: process.env.CLICKHOUSE_USER,
  password: process.env.CLICKHOUSE_PASSWORD,
});

async function createChannel(channelId, merkleRoot, initialTokenIds) {
    const initialTokenIdsJson = JSON.stringify(initialTokenIds);
    console.log(`Attempting to create channel with ID: ${channelId}`);
    console.log(`Merkle root: ${merkleRoot}`);
    console.log(`Initial token IDs: ${initialTokenIdsJson}`);
    
    const query = `
      INSERT INTO collections (collection_id, merkle_root, total_supply, next_token_number, initial_token_ids)
      VALUES ('${channelId}', '${merkleRoot}', 0, 1, '${initialTokenIdsJson}')
    `;
    
    console.log(`Executing query: ${query}`);
    
    try {
      const result = await client.command({
        query: query,
      });
      console.log(`Query executed successfully. Result:`, result);
      return result;
    } catch (error) {
      console.error(`Error executing query:`, error);
      throw error;
    }
  }

async function getChannelInfo(channelId) {
  const query = `SELECT * FROM collections WHERE collection_id = '${channelId}'`;
  const result = await client.query({
    query: query,
    format: 'JSONEachRow',
  });
  const rows = await result.json();
  return rows[0];
}

async function getInitialTokenIds(channelId) {
    // First, get the total supply for the channel
    const supplyQuery = `SELECT total_supply FROM collections WHERE collection_id = '${channelId}'`;
    const supplyResult = await client.query({
        query: supplyQuery,
        format: 'JSONEachRow',
    });
    const supplyRows = await supplyResult.json();
    
    if (!supplyRows.length || supplyRows[0].total_supply === 0) {
        console.log(`No tokens minted yet for channel ${channelId}`);
        return [];
    }

    // If total_supply > 0, query the tokens table
    const tokensQuery = `
        SELECT token_id 
        FROM tokens 
        WHERE collection_id = '${channelId}' 
        ORDER BY token_id
    `;
    const tokensResult = await client.query({
        query: tokensQuery,
        format: 'JSONEachRow',
    });
    const tokenRows = await tokensResult.json();

    // Extract and return the token IDs
    const tokenIds = tokenRows.map(row => row.token_id);
    console.log(`Found ${tokenIds.length} tokens for channel ${channelId}`);
    return tokenIds;
}

async function mintToken(channelId, tokenNumber, ownerId) {
  const tokenId = `${channelId}:${tokenNumber}`;
  const query = `
    INSERT INTO tokens (token_id, collection_id, owner_id)
    VALUES ('${tokenId}', '${channelId}', '${ownerId}')
  `;
  await client.command({
    query: query,
  });

  const updateQuery = `
    ALTER TABLE collections
    UPDATE total_supply = total_supply + 1, next_token_number = next_token_number + 1
    WHERE collection_id = '${channelId}'
  `;
  await client.command({
    query: updateQuery,
  });
}

async function getTokenOwner(channelId, tokenNumber) {
  const query = `
    SELECT owner_id FROM tokens
    WHERE collection_id = '${channelId}' AND token_id = ${tokenNumber}
  `;
  const result = await client.query({
    query: query,
    format: 'JSONEachRow',
  });
  const rows = await result.json();
  return rows[0]?.owner_id;
}

async function updateChannelTokenCount(channelId, count) {
    const query = `
        ALTER TABLE collections
        UPDATE total_supply = ${count}
        WHERE collection_id = '${channelId}'
    `;
    await client.command({query: query});
}

 async function getChannelTokenCount(channelId) {
    const query = `SELECT total_supply FROM collections WHERE collection_id = '${channelId}'`;
    const result = await client.query({query});
    const rows = await result.json();
    return rows[0]?.total_supply || 0;
}

async function getAllTokenIds(channelId) {
    const query = `SELECT token_id FROM tokens WHERE collection_id = '${channelId}' ORDER BY token_id`;
    const result = await client.query({
        query: query,
        format: 'JSONEachRow',
      });
      const rows = await result.json();
    console.log(rows.length);
    if (!rows.length) {
        console.log(`No tokens found for channel ${channelId}. This may be a new channel.`);
        return [];
    }
    
    return rows.map(row => row.token_id);
}

async function updateChannelMerkleRoot(channelId, newMerkleRoot) {
    console.log(`Updating Merkle root for channel ${channelId} to ${newMerkleRoot}`);
    
    const query = `
        ALTER TABLE collections
        UPDATE merkle_root = '${newMerkleRoot}'
        WHERE collection_id = '${channelId}'
    `;
    
    console.log(`Executing query: ${query}`);
    
    try {
        const result = await client.exec({query});
        console.log(`Query executed successfully. Result:`, result);
        
        // Verify the update
        const verifyQuery = `SELECT merkle_root FROM collections WHERE collection_id = '${channelId}'`;
        const verifyResult = await client.query({query: verifyQuery, format: 'JSONEachRow'});
        console.log(verifyQuery);
        const updatedRoot = await verifyResult.json();
        console.log(`Verified updated Merkle root:`, updatedRoot);
        
        if (updatedRoot[0]?.merkle_root !== newMerkleRoot) {
            console.warn(`Merkle root update mismatch. Expected: ${newMerkleRoot}, Actual: ${updatedRoot[0]?.merkle_root}`);
        }
        
        return result;
    } catch (error) {
        console.error(`Error executing query:`, error);
        throw error;
    }
}

async function transferToken(channelId, tokenNumber, newOwnerId) {
  const query = `
    ALTER TABLE tokens
    UPDATE owner_id = '${newOwnerId}'
    WHERE collection_id = '${channelId}' AND token_id = ${tokenNumber}
  `;
  await client.command({
    query: query,
  });
}

async function burnToken(channelId, tokenNumber) {
  const deleteQuery = `
    DELETE FROM tokens
    WHERE collection_id = '${channelId}' AND token_id = ${tokenNumber}
  `;
  await client.command({
    query: deleteQuery,
  });

  const updateQuery = `
    ALTER TABLE collections
    UPDATE total_supply = total_supply - 1
    WHERE collection_id = '${channelId}'
  `;
  await client.command({
    query: updateQuery,
  });
}

module.exports = {
  createChannel,
  getChannelInfo,
  getInitialTokenIds,
  mintToken,
  getTokenOwner,
  transferToken,
  burnToken,
  updateChannelTokenCount,
  getChannelTokenCount,
  getAllTokenIds,
  updateChannelMerkleRoot,
};