// Protocol 22 Soroban Contracts Verification Script
const { execSync } = require('child_process');

console.log('Verifying Soroban smart contracts for Protocol 22 compatibility...');

try {
  const output = execSync('cargo test --workspace', { stdio: 'pipe', encoding: 'utf8' });
  console.log('Test output:');
  console.log(output);
  console.log('\n[SUCCESS] All Soroban smart contracts passed test verification.');
} catch (err) {
  console.error('[ERROR] Verification failed:');
  console.error(err.stdout || err.message);
  process.exit(1);
}
