Certainly! I'll provide a step-by-step guide to deploy the entire setup, including the database, smart contract, and APIs. This guide assumes you're working with a NEAR testnet environment.

1. Deploy the Smart Contract:

   a. Install NEAR CLI if you haven't already:
   ```
   npm install -g near-cli
   ```

   b. Login to your NEAR account:
   ```
   near login
   ```

   c. Build the contract:
   ```
   cargo build --target wasm32-unknown-unknown --release
   ```

   d. Deploy the contract:
   ```
   near deploy --accountId YOUR_ACCOUNT_ID.testnet --wasmFile target/wasm32-unknown-unknown/release/shard_nfts.wasm
   ```

   e. Initialize the contract:
   ```
   near call YOUR_ACCOUNT_ID.testnet new '{"owner_id": "YOUR_ACCOUNT_ID.testnet"}' --accountId YOUR_ACCOUNT_ID.testnet
   ```

2. Set up the Database (ClickHouse):

   a. Install ClickHouse:
   ```
   sudo apt-get install clickhouse-server clickhouse-client
   ```

   b. Start ClickHouse server:
   ```
   sudo service clickhouse-server start
   ```

   c. Create the database and tables:
   ```sql
   clickhouse-client --multiline <<EOF
   CREATE DATABASE IF NOT EXISTS nft_db;

   USE nft_db;

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
   EOF
   ```

3. Set up the Middleware:

   a. Create a new directory for the middleware:
   ```
   mkdir shard_nfts_middleware && cd shard_nfts_middleware
   ```

   b. Initialize a new Node.js project:
   ```
   npm init -y
   ```

   c. Install required dependencies:
   ```
   npm install near-api-js merkletreejs crypto-js express dotenv clickhouse-driver
   ```

   d. Create a `.env` file with your configuration:
   ```
   NEAR_NETWORK_ID=testnet
   NEAR_NODE_URL=https://rpc.testnet.near.org
   NEAR_WALLET_URL=https://wallet.testnet.near.org
   NEAR_HELPER_URL=https://helper.testnet.near.org
   NEAR_CONTRACT_NAME=YOUR_ACCOUNT_ID.testnet
   CLICKHOUSE_HOST=localhost
   CLICKHOUSE_PORT=9000
   CLICKHOUSE_DB=nft_db
   ```

   e. Create an `index.js` file with the following content:

   ```javascript
   require('dotenv').config();
   const express = require('express');
   const nearAPI = require('near-api-js');
   const { MerkleTree } = require('merkletreejs');
   const crypto = require('crypto');
   const { ClickHouse } = require('clickhouse-driver');

   const app = express();
   app.use(express.json());

   // Initialize NEAR connection
   const near = await nearAPI.connect({
     networkId: process.env.NEAR_NETWORK_ID,
     nodeUrl: process.env.NEAR_NODE_URL,
     walletUrl: process.env.NEAR_WALLET_URL,
     helperUrl: process.env.NEAR_HELPER_URL,
   });

   const account = await near.account(process.env.NEAR_CONTRACT_NAME);
   const contract = new nearAPI.Contract(account, process.env.NEAR_CONTRACT_NAME, {
     viewMethods: ['get_channel_info', 'is_minted', 'get_next_token_number'],
     changeMethods: ['create_channel', 'mint', 'update_merkle_root'],
   });

   // Initialize ClickHouse connection
   const clickhouse = new ClickHouse({
     host: process.env.CLICKHOUSE_HOST,
     port: process.env.CLICKHOUSE_PORT,
     database: process.env.CLICKHOUSE_DB,
   });

   // Include the TokenManager class and other functions from middle.js here

   // API endpoints
   app.post('/create-channel', async (req, res) => {
     const { channelId, initialTokenIds } = req.body;
     try {
       const tokenManager = await createChannel(channelId, initialTokenIds);
       res.json({ success: true, message: 'Channel created successfully' });
     } catch (error) {
       res.status(500).json({ success: false, error: error.message });
     }
   });

   app.post('/mint', async (req, res) => {
     const { channelId } = req.body;
     try {
       const tokenManager = new TokenManager(channelId); // You might want to store this in a database
       await mintNextToken(channelId, tokenManager);
       res.json({ success: true, message: 'Token minted successfully' });
     } catch (error) {
       res.status(500).json({ success: false, error: error.message });
     }
   });

   // Add more API endpoints as needed

   const PORT = process.env.PORT || 3000;
   app.listen(PORT, () => console.log(`Server running on port ${PORT}`));
   ```

   f. Start the middleware server:
   ```
   node index.js
   ```

4. Set up a Cron Job for Syncing:

   Create a script `sync.js` to sync the on-chain data with the off-chain database:

   ```javascript
   // Include necessary imports and initializations

   async function syncData() {
     // Fetch all channels from the contract
     const channels = await contract.get_all_channels();

     for (const channelId of channels) {
       const channelInfo = await contract.get_channel_info({ channel_id: channelId });
       
       // Update channel info in ClickHouse
       await clickhouse.query(`
         INSERT INTO channels (channel_id, merkle_root, total_supply, next_token_number, updated_at)
         VALUES (?, ?, ?, ?, now())
       `, [channelId, channelInfo.merkle_root, channelInfo.total_supply, channelInfo.next_token_number]);

       // Fetch and update token info
       for (let i = 1; i < channelInfo.next_token_number; i++) {
         const tokenId = `${channelId}:${i}`;
         if (await contract.is_minted({ token_id: tokenId })) {
           // Fetch owner from the contract (you might need to implement this method)
           const owner = await contract.get_token_owner({ token_id: tokenId });
           
           await clickhouse.query(`
             INSERT INTO tokens (token_id, channel_id, owner_id, token_number)
             VALUES (?, ?, ?, ?)
           `, [tokenId, channelId, owner, i]);
         }
       }
     }
   }

   syncData().then(() => console.log('Sync completed')).catch(console.error);
   ```

   Set up a cron job to run this script periodically:

   ```
   crontab -e
   ```

   Add the following line to run the sync every hour:

   ```
   0 * * * * /usr/bin/node /path/to/your/sync.js >> /path/to/logfile.log 2>&1
   ```

5. Testing:

   You can now test your setup by creating a channel and minting tokens:

   ```
   curl -X POST http://localhost:3000/create-channel -H "Content-Type: application/json" -d '{"channelId": "test_channel", "initialTokenIds": [1,2,3,4,5]}'

   curl -X POST http://localhost:3000/mint -H "Content-Type: application/json" -d '{"channelId": "test_channel"}'
   ```

Remember to implement proper error handling, logging, and security measures in a production environment. This setup provides a basic framework that you can expand upon based on your specific requirements.



You're right to question this. For a high-frequency minting scenario like 10 mints per minute, relying solely on hourly cron job syncing might not be sufficient. Let's consider some improvements to make the system more real-time and robust:

1. Real-time Updates:
   Instead of relying solely on periodic syncing, implement real-time updates in your middleware:

   ```javascript
   async function mintNextToken(channelId, tokenManager) {
     // ... existing code ...

     await contract.mint(
       {
         channel_id: channelId,
         proof: proof,
       },
       300000000000000,
       utils.format.parseNearAmount("0.1")
     );

     // Add this: Update the database immediately after successful mint
     await updateDatabaseAfterMint(channelId, nextTokenNumber, owner_id);

     console.log(`Token ${channelId}:${nextTokenNumber} minted`);
   }

   async function updateDatabaseAfterMint(channelId, tokenNumber, ownerId) {
     const tokenId = `${channelId}:${tokenNumber}`;
     await clickhouse.query(`
       INSERT INTO tokens (token_id, channel_id, owner_id, token_number, minted_at)
       VALUES (?, ?, ?, ?, now())
     `, [tokenId, channelId, ownerId, tokenNumber]);

     await clickhouse.query(`
       UPDATE channels
       SET total_supply = total_supply + 1,
           next_token_number = next_token_number + 1,
           updated_at = now()
       WHERE channel_id = ?
     `, [channelId]);
   }
   ```

2. Event Listening:
   Implement NEAR event listening to catch any mints that might have been missed:

   ```javascript
   const nearAPI = require('near-api-js');

   async function listenForMintEvents() {
     const provider = new nearAPI.providers.JsonRpcProvider(process.env.NEAR_NODE_URL);
     
     const blockCallback = async (error, block) => {
       if (error) {
         console.error("Error processing block:", error);
         return;
       }

       for (let outcome of block.receipts_outcome) {
         const logs = outcome.outcome.logs;
         for (let log of logs) {
           if (log.startsWith("EVENT_JSON:")) {
             const eventData = JSON.parse(log.slice(11));
             if (eventData.event === "nft_mint") {
               for (let mint of eventData.data) {
                 await updateDatabaseAfterMint(
                   mint.token_ids[0].split(':')[0], // channel_id
                   parseInt(mint.token_ids[0].split(':')[1]), // token_number
                   mint.owner_id
                 );
               }
             }
           }
         }
       }
     };

     while (true) {
       try {
         await provider.sendJsonRpc("EXPERIMENTAL_changes", {
           finality: "final",
           account_ids: [process.env.NEAR_CONTRACT_NAME],
           subscribe: true
         }, blockCallback);
       } catch (error) {
         console.error("Error in event listener:", error);
         // Wait before trying to reconnect
         await new Promise(resolve => setTimeout(resolve, 1000));
       }
     }
   }

   // Start listening for events
   listenForMintEvents();
   ```

3. Periodic Reconciliation:
   Keep the hourly cron job as a fallback mechanism to catch any discrepancies:

   ```javascript
   async function reconcileData() {
     const channels = await contract.get_all_channels();

     for (const channelId of channels) {
       const channelInfo = await contract.get_channel_info({ channel_id: channelId });
       
       // Update channel info in ClickHouse
       await clickhouse.query(`
         INSERT INTO channels (channel_id, merkle_root, total_supply, next_token_number, updated_at)
         VALUES (?, ?, ?, ?, now())
         ON DUPLICATE KEY UPDATE
           merkle_root = VALUES(merkle_root),
           total_supply = VALUES(total_supply),
           next_token_number = VALUES(next_token_number),
           updated_at = VALUES(updated_at)
       `, [channelId, channelInfo.merkle_root, channelInfo.total_supply, channelInfo.next_token_number]);

       // Fetch and update token info
       const dbTokens = await clickhouse.query(`SELECT token_number FROM tokens WHERE channel_id = ?`, [channelId]);
       const dbTokenSet = new Set(dbTokens.map(t => t.token_number));

       for (let i = 1; i < channelInfo.next_token_number; i++) {
         if (!dbTokenSet.has(i) && await contract.is_minted({ token_id: `${channelId}:${i}` })) {
           const owner = await contract.get_token_owner({ token_id: `${channelId}:${i}` });
           await updateDatabaseAfterMint(channelId, i, owner);
         }
       }
     }
   }
   ```

4. Database Optimizations:
   For high-frequency writes, consider:
   - Using ClickHouse's buffer engine for the `tokens` table to batch inserts.
   - Implementing a queue system (like Redis) to handle database updates asynchronously.

5. Monitoring and Alerts:
   Implement monitoring to alert you of any discrepancies between on-chain and off-chain data.

6. Rate Limiting:
   Implement rate limiting in your API to ensure it can handle the load and prevent abuse.

With these improvements:
1. Most mints will be recorded in real-time through the middleware.
2. Any missed mints will be caught by the event listener.
3. The hourly reconciliation will catch any remaining discrepancies.

This multi-layered approach should be able to handle 10 mints per minute while maintaining data consistency between your contract and database. However, always monitor the system closely, especially during high-load periods, and be prepared to scale your infrastructure if needed.