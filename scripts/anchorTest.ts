import { execSync } from 'child_process'

function setupTestEnvironment() {
  const env = { ...process.env, FEATURE: 'localnet' }
  execSync('pnpm run anchor build && pnpm run generate-clients && pnpm anchor test --skip-build', {
    stdio: 'inherit',
    env,
  })
}

setupTestEnvironment()
