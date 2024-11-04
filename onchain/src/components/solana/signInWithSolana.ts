import bs58 from 'bs58'
import type { Adapter } from '@solana/wallet-adapter-base'
import { createSignInMessage } from '@solana/wallet-standard-util'
import type { SolanaSignInInput, SolanaSignInOutput } from '@solana/wallet-standard-features'

export async function signInWithSolana(adapter: Adapter) {
  if ('signIn' in adapter) {
    const signInInputResponse = await fetch(`/api/auth/signin/`)
    const signInInput: SolanaSignInInput = await signInInputResponse.json()
    const signInOutput: SolanaSignInOutput = await adapter.signIn(signInInput)
    const verifyResult = await doVerify(signInInput, signInOutput)
    if (!verifyResult) throw new Error('Failed to sign in.')
    return false
  } else {
    return true
  }
}

export async function manualSignIn(wallet: Adapter) {
  if (wallet.publicKey && 'signMessage' in wallet) {
    const signInInputResponse = await fetch(`/api/auth/signin/`)
    const signInInput: SolanaSignInInput = await signInInputResponse.json()
    const domain = signInInput.domain || window.location.host
    const address = wallet.publicKey.toBase58()
    const signedMessage = createSignInMessage({ ...signInInput, domain, address })
    const signature = await wallet.signMessage(signedMessage)
    const signInOutput = {
      account: {
        address: wallet.publicKey.toBase58(),
        publicKey: wallet.publicKey.toBytes(),
        chains: [],
        features: [],
      },
      signedMessage,
      signature,
    }
    const verifyResult = await doVerify(signInInput, signInOutput)
    if (!verifyResult) throw new Error('Failed to sign in.')
  }
}

async function doVerify(signInput: SolanaSignInInput, signInOutput: SolanaSignInOutput) {
  const verifyResponse = await fetch(`/api/auth/verify/`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      publicKey: bs58.encode(signInOutput.account.publicKey),
      signedMessage: Buffer.from(signInOutput.signedMessage).toString('utf-8'),
      signature: bs58.encode(signInOutput.signature),
      issuedAt: signInput.issuedAt,
    }),
  })
  return await verifyResponse.json()
}
