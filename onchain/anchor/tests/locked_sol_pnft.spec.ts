import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import {
  unwrapOption,
  createSignerFromKeypair,
  keypairIdentity,
  sol,
  publicKey,
  generateSigner,
} from '@metaplex-foundation/umi'
import { mplTokenMetadata, fetchMetadata } from '@metaplex-foundation/mpl-token-metadata'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { Keypair } from '@solana/web3.js'
import { SendTransactionError, PublicKey as web3PublicKey } from '@solana/web3.js'
import { getAssociatedTokenAddress, AccountLayout } from '@solana/spl-token'
import { initializeMasterEdition, mintPnft } from '../../clients/generated/umi/src/instructions'
import fs from 'fs'
import * as path from 'path'
import { transactionBuilder } from '@metaplex-foundation/umi'
import { setComputeUnitLimit, setComputeUnitPrice } from '@metaplex-foundation/mpl-toolbox'

const umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })
umi.use(mplTokenMetadata())

const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')
const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))
const updateAuthority = createSignerFromKeypair(umi, {
  publicKey: publicKey(keyPair.publicKey.toString()),
  secretKey: keyPair.secretKey,
})

const payer = generateSigner(umi)
umi.use(keypairIdentity(payer))

async function setupInitializeMasterEdition() {
  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
  const metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

  // Get master mint PDA first since we need it for master_state
  const [masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])

  // Get master state PDA using correct seeds from Rust code
  const [masterState] = umi.eddsa.findPda(publicKey(program.programId), [
    Buffer.from('master'),
    publicKeySerializer().serialize(masterMint),
  ])

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

  // Get delegate authority PDA
  const [delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
    Buffer.from('collection_delegate'),
    publicKeySerializer().serialize(masterMint),
  ])

  // Get collection authority record PDA
  const [collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(masterMint),
    Buffer.from('collection_authority'),
    publicKeySerializer().serialize(delegateAuthority),
  ])

  const associatedTokenAccount = await getAssociatedTokenAddress(
    new web3PublicKey(masterMint.toString()),
    new web3PublicKey(updateAuthority.publicKey.toString()),
    true,
  )

  const authorityToken = publicKey(associatedTokenAccount.toString())

  console.log('\nDerived Program PDAs:')
  console.log('Master Mint:', masterMint.toString())
  console.log('Master State:', masterState.toString(), '(derived from ["master", master_mint])')
  console.log('Master Metadata:', masterMetadata.toString())
  console.log('Master Edition:', masterEdition.toString())
  console.log('Authority Token:', authorityToken.toString())
  console.log('Collection Authority Record:', collectionAuthorityRecord.toString())

  return {
    masterMint,
    masterState,
    masterMetadata,
    masterEdition,
    authorityToken,
    collectionAuthorityRecord,
    delegateAuthority,
  }
}

async function setupMintPnft() {
  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
  const metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

  // Create new mint for the NFT
  const mint = generateSigner(umi)
  console.log('New Mint:', mint.publicKey.toString())

  // Get master mint PDA
  const [masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])

  // Get master state PDA
  const [masterState] = umi.eddsa.findPda(publicKey(program.programId), [
    Buffer.from('master'),
    publicKeySerializer().serialize(masterMint),
  ])

  // Get delegate authority PDA
  const [delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
    Buffer.from('collection_delegate'),
    publicKeySerializer().serialize(masterMint),
  ])

  // Get collection authority record PDA
  const [collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(masterMint),
    Buffer.from('collection_authority'),
    publicKeySerializer().serialize(delegateAuthority),
  ])

  // Get collection metadata PDA
  const [collectionMetadata] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(masterMint),
  ])

  // Get collection master edition PDA
  const [collectionMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(masterMint),
    Buffer.from('edition'),
  ])

  // Get NFT metadata PDA
  const [metadata] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(mint.publicKey),
  ])

  // Get NFT master edition PDA
  const [masterEdition] = umi.eddsa.findPda(metadataProgramId, [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(metadataProgramId),
    publicKeySerializer().serialize(mint.publicKey),
    Buffer.from('edition'),
  ])

  // Get token account
  const tokenAccount = await getAssociatedTokenAddress(
    new web3PublicKey(mint.publicKey.toString()),
    new web3PublicKey(payer.publicKey.toString()),
  )

  return {
    payer,
    masterState,
    masterMint,
    collectionMetadata,
    collectionMasterEdition,
    metadata,
    masterEdition,
    mint,
    delegateAuthority,
    collectionAuthorityRecord,
    tokenAccount: publicKey(tokenAccount.toString()),
  }
}

// describe('initializeMasterEdition Instruction', () => {
//   it('should fail to create a master edition with unauthorized update authority', async () => {
//     await umi.rpc.airdrop(payer.publicKey, sol(1))

//     const { masterMint, masterMetadata, masterEdition, authorityToken } = await setupInitializeMasterEdition()

//     const unauthorizedAuthority = generateSigner(umi)
//     try {
//       await initializeMasterEdition(umi, {
//         payer,
//         masterMint,
//         masterMetadata,
//         masterEdition,
//         updateAuthority: unauthorizedAuthority,
//         updateAuthorityToken: authorityToken,
//       }).sendAndConfirm(umi)
//     } catch (error) {
//       if (error instanceof SendTransactionError) {
//         expect(error.message).toContain('Invalid update authority')
//       } else {
//         throw error
//       }
//     }
//   })

//   it('should create a master edition, and associated account with balance 1', async () => {
//     await umi.rpc.airdrop(payer.publicKey, sol(1))

//     const { masterMint, masterMetadata, masterEdition, associatedTokenAccount, authorityToken } =
//       await setupInitializeMasterEdition()

//     await initializeMasterEdition(umi, {
//       payer,
//       masterMint,
//       masterMetadata,
//       masterEdition,
//       updateAuthority,
//       updateAuthorityToken: authorityToken,
//     }).sendAndConfirm(umi)

//     const metadataAccount = await umi.rpc.getAccount(masterMetadata)
//     expect(metadataAccount.exists).toBe(true)

//     const metadata = await fetchMetadata(umi, masterMetadata)
//     expect(metadata.name).toBe('Locked SOL NFT')
//     expect(metadata.symbol).toBe('LSOL')
//     expect(metadata.uri).toBe('https://api.locked-sol.com/metadata/initial.json')
//     expect(metadata.sellerFeeBasisPoints).toBe(0)
//     expect(metadata.isMutable).toBe(true)
//     expect(metadata.updateAuthority.toString()).toBe(updateAuthority.publicKey.toString())

//     const tokenAccountInfo = await umi.rpc.getAccount(publicKey(associatedTokenAccount.toString()))
//     expect(tokenAccountInfo).toBeTruthy()

//     if (!tokenAccountInfo.exists) {
//       throw new Error('Token account does not exist')
//     }

//     const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
//     const balance = BigInt(decodedAccount.amount.toString())
//     expect(balance).toBe(BigInt(1))
//   })

//   it('should fail when trying to create a master edition twice', async () => {
//     const { masterMint, masterMetadata, masterEdition, authorityToken } = await setupInitializeMasterEdition()

//     try {
//       await initializeMasterEdition(umi, {
//         payer,
//         masterMint,
//         masterMetadata,
//         masterEdition,
//         updateAuthority,
//         updateAuthorityToken: authorityToken,
//       }).sendAndConfirm(umi)
//     } catch (error) {
//       if (error instanceof SendTransactionError) {
//         expect(error.message).toContain('Transaction simulation failed')
//         expect(error.logs).toContainEqual(expect.stringContaining('Allocate: account Address'))
//         expect(error.logs).toContainEqual(expect.stringContaining('already in use'))
//       } else {
//         throw error
//       }
//     }
//   })
// })

describe('mintPnft Instruction', () => {
  beforeAll(async () => {
    await umi.rpc.airdrop(payer.publicKey, sol(2))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(2))

    const {
      masterMint,
      masterState,
      masterMetadata,
      masterEdition,
      authorityToken,
      collectionAuthorityRecord,
      delegateAuthority,
    } = await setupInitializeMasterEdition()

    console.log('Initializing master edition...')

    console.log('Initializing master edition...')
    console.log('Master Mint:', masterMint.toString())
    console.log('Master Metadata:', masterMetadata.toString())
    console.log('Master Edition:', masterEdition.toString())
    console.log('Authority Token:', authorityToken.toString())
    console.log('Collection Authority Record:', collectionAuthorityRecord.toString())

    // Create the master edition transaction with exactly the accounts the UMI client expects
    const tx = transactionBuilder()
      .add(setComputeUnitLimit(umi, { units: 400_000 }))
      .add(
        initializeMasterEdition(umi, {
          payer,
          masterState,
          masterMint,
          masterMetadata,
          masterEdition,
          updateAuthority,
          updateAuthorityToken: authorityToken,
          collectionAuthorityRecord,
          delegateAuthority,
        }),
      )

    try {
      await tx.sendAndConfirm(umi)
    } catch (error) {
      console.error('Error during initialization:', error)
      const accounts = {
        masterMint: await umi.rpc.getAccount(masterMint),
        masterMetadata: await umi.rpc.getAccount(masterMetadata),
        masterEdition: await umi.rpc.getAccount(masterEdition),
        authorityToken: await umi.rpc.getAccount(authorityToken),
        collectionAuthorityRecord: await umi.rpc.getAccount(collectionAuthorityRecord),
      }
      console.log('Account states:', accounts)
      throw error
    }

    // Verify all required accounts exist
    console.log('Verifying master mint account...')
    const masterMintAccount = await umi.rpc.getAccount(masterMint)

    console.log('\nMaster Mint Account Details:')
    console.log('Exists:', masterMintAccount.exists)

    if (!masterMintAccount.exists) {
      throw new Error('Master mint account does not exist after initialization')
    }

    // Now we can safely access owner and data since we've confirmed the account exists
    console.log('Owner:', masterMintAccount.owner.toString())
    console.log('Data length:', masterMintAccount.data.length)

    console.log('Verifying master metadata account...')
    const masterMetadataAccount = await umi.rpc.getAccount(masterMetadata)
    console.log('Master metadata account exists:', masterMetadataAccount.exists)

    console.log('Verifying master edition account...')
    const masterEditionAccount = await umi.rpc.getAccount(masterEdition)
    console.log('Master edition account exists:', masterEditionAccount.exists)

    if (!masterMintAccount.exists || !masterMetadataAccount.exists || !masterEditionAccount.exists) {
      throw new Error('Required accounts not properly initialized!')
    }

    console.log('Master mint initialized successfully!')
  })

  it('should successfully mint a pNFT and verify collection', async () => {
    await umi.rpc.airdrop(payer.publicKey, sol(3))

    const {
      masterState,
      masterMint,
      collectionMetadata,
      collectionMasterEdition,
      metadata,
      masterEdition,
      mint,
      delegateAuthority,
      collectionAuthorityRecord,
      tokenAccount,
    } = await setupMintPnft()

    const masterStateAccount = await umi.rpc.getAccount(masterState)
    if (!masterStateAccount.exists) {
      throw new Error('Master state account does not exist')
    }

    // Master state has this layout in Rust:
    // pub struct MasterState {
    //     pub master_mint: Pubkey,  // 32 bytes
    //     pub total_minted: u64,    // 8 bytes
    // }
    // Plus 8 bytes discriminator
    const discriminator = masterStateAccount.data.slice(0, 8)
    const masterMintInState = new web3PublicKey(masterStateAccount.data.slice(8, 40))
    const totalMinted = masterStateAccount.data.slice(40, 48)

    console.log('Master State Data:')
    console.log('Discriminator:', discriminator)
    console.log('Master Mint in State:', masterMintInState.toString())
    console.log('Total Minted:', new anchor.BN(totalMinted).toString())

    // Get expected master mint for comparison
    const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
    const [expectedMasterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])
    console.log('Expected Master Mint:', expectedMasterMint.toString())

    if (masterMintInState.toString() !== expectedMasterMint.toString()) {
      throw new Error(
        `Master mint mismatch in state. Expected ${expectedMasterMint.toString()}, got ${masterMintInState.toString()}`,
      )
    }

    const ix = mintPnft(umi, {
      payer,
      masterState,
      masterMint,
      collectionMetadata,
      collectionMasterEdition,
      metadata,
      masterEdition,
      mint,
      delegateAuthority,
      collectionAuthorityRecord,
      tokenAccount,
    })

    // Set the desired compute unit limit
    const computeUnitLimit = 400_000 // Adjust this value based on your transaction's requirements

    // Optionally, set a compute unit price to prioritize your transaction
    const computeUnitPrice = 1 // Price per compute unit in micro-lamports

    // Build the transaction with the compute budget adjustments
    transactionBuilder()
      .add(setComputeUnitLimit(umi, { units: computeUnitLimit }))
      .add(setComputeUnitPrice(umi, { microLamports: computeUnitPrice }))
      .add(ix)
      .sendAndConfirm(umi)

    const tokenAccountInfo = await umi.rpc.getAccount(tokenAccount)
    if (!tokenAccountInfo.exists) {
      throw new Error('Token account does not exist')
    }
    const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
    expect(BigInt(decodedAccount.amount.toString())).toBe(BigInt(1))

    const nftMetadata = await fetchMetadata(umi, metadata)
    expect(nftMetadata.collection).not.toBeNull()

    const collection = unwrapOption(nftMetadata.collection)
    if (collection) {
      expect(collection.key.toString()).toBe(masterState.toString())
      expect(collection.verified).toBe(true)
    } else {
      throw new Error('Collection is None')
    }
  })
})
