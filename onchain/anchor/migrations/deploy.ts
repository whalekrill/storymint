// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.
import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { createSignerFromKeypair, keypairIdentity, publicKey, generateSigner } from '@metaplex-foundation/umi'
import { mplTokenMetadata } from '@metaplex-foundation/mpl-token-metadata'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { Keypair } from '@solana/web3.js'
import { PublicKey as web3PublicKey } from '@solana/web3.js'
import { getAssociatedTokenAddress, AccountLayout } from '@solana/spl-token'
import { initializeMasterEdition } from '../../clients/generated/umi/src/instructions'
import * as fs from 'fs'
import * as path from 'path'

module.exports = async function (provider: anchor.Provider) {
  const umi = createUmi(provider.connection.rpcEndpoint, {
    commitment: provider.connection.commitment,
  })
  umi.use(mplTokenMetadata())

  const keypairPath = path.join(__dirname, '../keys/update-authority-devnet.json')
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
  const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))

  const updateAuthority = createSignerFromKeypair(umi, {
    publicKey: publicKey(keyPair.publicKey.toString()),
    secretKey: keyPair.secretKey,
  })
  const payer = generateSigner(umi)
  umi.use(keypairIdentity(payer))

  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
  const metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

  console.log('Initializing master edition...')

  const [masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])

  const [masterMetadata] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(masterMint),
  ])

  const [masterEdition] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(masterMint),
    Buffer.from('edition'),
  ])

  const associatedTokenAccount = await getAssociatedTokenAddress(
    new web3PublicKey(masterMint.toString()),
    new web3PublicKey(updateAuthority.publicKey.toString()),
    true,
  )
  const authorityToken = publicKey(associatedTokenAccount.toString())

  await initializeMasterEdition(umi, {
    payer,
    masterMint,
    masterMetadata,
    masterEdition,
    updateAuthority,
    authorityToken,
  }).sendAndConfirm(umi)

  const tokenAccountInfo = await umi.rpc.getAccount(publicKey(associatedTokenAccount.toString()))
  if (!tokenAccountInfo.exists) {
    throw new Error('Token account does not exist')
  }
  const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
  const balance = BigInt(decodedAccount.amount.toString())

  if (balance == BigInt(1)) {
    console.log('Master edition initialized successfully!')
  } else {
    throw new Error(`Expected balance of 1, got ${balance}`)
  }
}
