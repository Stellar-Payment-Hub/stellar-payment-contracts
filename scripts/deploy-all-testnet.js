const StellarSdk = require('@stellar/stellar-sdk');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const rpc = new StellarSdk.rpc.Server('https://soroban-testnet.stellar.org');
const networkPassphrase = StellarSdk.Networks.TESTNET;

async function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function sendAndPoll(tx, keypair) {
  const prepared = await rpc.prepareTransaction(tx);
  prepared.sign(keypair);
  const sendRes = await rpc.sendTransaction(prepared);
  if (sendRes.status === 'ERROR') {
    throw new Error(`Send error: ${JSON.stringify(sendRes)}`);
  }
  const hash = sendRes.hash;
  console.log(`  Tx sent: ${hash} (status: ${sendRes.status})`);

  let poll = await rpc.getTransaction(hash);
  let count = 0;
  while (poll.status === 'NOT_FOUND' && count < 35) {
    await sleep(1500);
    poll = await rpc.getTransaction(hash);
    count++;
  }

  if (poll.status !== 'SUCCESS') {
    throw new Error(`Transaction failed with status ${poll.status}: ${JSON.stringify(poll)}`);
  }
  console.log(`  Tx confirmed in ledger ${poll.ledger}!`);
  return { hash, result: poll };
}

async function main() {
  console.log('=== DEPLOYING REAL STELLAR PAYMENT HUB CONTRACTS TO TESTNET ===');
  
  // 1. Setup Deployer Keypair
  const deployer = StellarSdk.Keypair.random();
  console.log(`1. Generated Deployer Account: ${deployer.publicKey()}`);
  console.log(`   Secret Seed: ${deployer.secret()}`);

  console.log('2. Requesting Friendbot funding...');
  await rpc.requestAirdrop(deployer.publicKey());
  await sleep(4500);

  let account = await rpc.getAccount(deployer.publicKey());
  console.log(`   Account balance active, sequence: ${account.sequence}`);

  // 3. Upload PaymentRegistry WASM
  console.log('\n3. Reading and uploading PaymentRegistry WASM...');
  const registryWasmPath = path.resolve(__dirname, '../target/wasm32v1-none/release/payment_registry.wasm');
  const registryWasm = fs.readFileSync(registryWasmPath);
  const registryWasmHash = crypto.createHash('sha256').update(registryWasm).digest();
  console.log(`   WASM file size: ${registryWasm.length} bytes`);
  console.log(`   Calculated WASM SHA256: ${registryWasmHash.toString('hex')}`);

  let uploadRegistryOp = StellarSdk.Operation.uploadContractWasm({ wasm: registryWasm });
  let tx = new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
    .addOperation(uploadRegistryOp)
    .setTimeout(60)
    .build();

  await sendAndPoll(tx, deployer);
  console.log('   PaymentRegistry WASM uploaded successfully.');

  // Refresh account sequence
  account = await rpc.getAccount(deployer.publicKey());

  // 4. Create PaymentRegistry Contract Instance
  console.log('\n4. Creating PaymentRegistry contract instance...');
  let createRegistryOp = StellarSdk.Operation.createCustomContract({
    address: StellarSdk.Address.fromString(deployer.publicKey()),
    wasmHash: registryWasmHash
  });

  let simTx = new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
    .addOperation(createRegistryOp)
    .setTimeout(60)
    .build();
  let simRes = await rpc.simulateTransaction(simTx);
  const registryContractAddress = StellarSdk.Address.fromScVal(simRes.result.retval).toString();
  console.log(`   PaymentRegistry Contract Address: ${registryContractAddress}`);

  let createRegistryRes = await sendAndPoll(simTx, deployer);
  console.log(`   PaymentRegistry deployed on ledger! Tx: ${createRegistryRes.hash}`);

  // Refresh account sequence
  account = await rpc.getAccount(deployer.publicKey());

  // 5. Initialize PaymentRegistry
  console.log('\n5. Initializing PaymentRegistry contract...');
  const registryContract = new StellarSdk.Contract(registryContractAddress);
  let initRegistryOp = registryContract.call(
    'initialize',
    new StellarSdk.Address(deployer.publicKey()).toScVal()
  );
  tx = new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
    .addOperation(initRegistryOp)
    .setTimeout(60)
    .build();
  await sendAndPoll(tx, deployer);
  console.log('   PaymentRegistry initialized with admin.');

  // Refresh account sequence
  account = await rpc.getAccount(deployer.publicKey());

  // 6. Upload SettlementRouter WASM
  console.log('\n6. Reading and uploading SettlementRouter WASM...');
  const routerWasmPath = path.resolve(__dirname, '../target/wasm32v1-none/release/settlement_router.wasm');
  const routerWasm = fs.readFileSync(routerWasmPath);
  const routerWasmHash = crypto.createHash('sha256').update(routerWasm).digest();
  console.log(`   WASM file size: ${routerWasm.length} bytes`);
  console.log(`   Calculated WASM SHA256: ${routerWasmHash.toString('hex')}`);

  let uploadRouterOp = StellarSdk.Operation.uploadContractWasm({ wasm: routerWasm });
  tx = new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
    .addOperation(uploadRouterOp)
    .setTimeout(60)
    .build();

  await sendAndPoll(tx, deployer);
  console.log('   SettlementRouter WASM uploaded successfully.');

  // Refresh account sequence
  account = await rpc.getAccount(deployer.publicKey());

  // 7. Create SettlementRouter Contract Instance
  console.log('\n7. Creating SettlementRouter contract instance...');
  let createRouterOp = StellarSdk.Operation.createCustomContract({
    address: StellarSdk.Address.fromString(deployer.publicKey()),
    wasmHash: routerWasmHash
  });

  simTx = new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
    .addOperation(createRouterOp)
    .setTimeout(60)
    .build();
  simRes = await rpc.simulateTransaction(simTx);
  const routerContractAddress = StellarSdk.Address.fromScVal(simRes.result.retval).toString();
  console.log(`   SettlementRouter Contract Address: ${routerContractAddress}`);

  let createRouterRes = await sendAndPoll(simTx, deployer);
  console.log(`   SettlementRouter deployed on ledger! Tx: ${createRouterRes.hash}`);

  // Refresh account sequence
  account = await rpc.getAccount(deployer.publicKey());

  // 8. Initialize SettlementRouter with admin and registry contract
  console.log('\n8. Initializing SettlementRouter contract...');
  const routerContract = new StellarSdk.Contract(routerContractAddress);
  let initRouterOp = routerContract.call(
    'initialize',
    new StellarSdk.Address(deployer.publicKey()).toScVal(),
    StellarSdk.Address.fromString(registryContractAddress).toScVal()
  );
  tx = new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
    .addOperation(initRouterOp)
    .setTimeout(60)
    .build();
  await sendAndPoll(tx, deployer);
  console.log('   SettlementRouter initialized with target registry.');

  // 9. Verification & Health Read
  console.log('\n9. Verifying on-chain state...');
  const registryStatusSim = await rpc.simulateTransaction(
    new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
      .addOperation(registryContract.call('status'))
      .setTimeout(60)
      .build()
  );
  console.log('   PaymentRegistry status read:', registryStatusSim.result.retval);

  const routerStatusSim = await rpc.simulateTransaction(
    new StellarSdk.TransactionBuilder(account, { fee: '100000', networkPassphrase })
      .addOperation(routerContract.call('status'))
      .setTimeout(60)
      .build()
  );
  console.log('   SettlementRouter status read:', routerStatusSim.result.retval);

  // 10. Write deployment metadata
  const deploymentData = {
    network: 'testnet',
    rpcUrl: 'https://soroban-testnet.stellar.org',
    networkPassphrase: StellarSdk.Networks.TESTNET,
    deployer: deployer.publicKey(),
    contracts: {
      PaymentRegistry: {
        contractAddress: registryContractAddress,
        wasmHash: registryWasmHash.toString('hex'),
        deploymentTransaction: createRegistryRes.hash,
        deploymentDate: new Date().toISOString()
      },
      SettlementRouter: {
        contractAddress: routerContractAddress,
        wasmHash: routerWasmHash.toString('hex'),
        deploymentTransaction: createRouterRes.hash,
        deploymentDate: new Date().toISOString(),
        targetRegistry: registryContractAddress
      }
    }
  };

  const deploymentPath = path.resolve(__dirname, '../deployments/testnet.json');
  fs.writeFileSync(deploymentPath, JSON.stringify(deploymentData, null, 2));
  console.log(`\nDeployment metadata saved to ${deploymentPath}`);

  console.log('\n================ DEPLOYMENT COMPLETE ================');
  console.log(`PaymentRegistry Contract:  ${registryContractAddress}`);
  console.log(`SettlementRouter Contract: ${routerContractAddress}`);
  console.log(`Deployer / Admin:          ${deployer.publicKey()}`);
  console.log('=====================================================');
}

main().catch((err) => {
  console.error('\nDeployment failed:', err);
  process.exit(1);
});
