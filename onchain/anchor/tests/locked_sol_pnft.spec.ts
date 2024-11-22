import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import {
  Umi,
  PublicKey,
  KeypairSigner,
  unwrapOption,
  createSignerFromKeypair,
  keypairIdentity,
  sol,
  publicKey,
  generateSigner,
  none,
  some,
} from '@metaplex-foundation/umi'
import { mplTokenMetadata, fetchMetadata } from '@metaplex-foundation/mpl-token-metadata'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { SendTransactionError, Keypair, PublicKey as web3PublicKey } from '@solana/web3.js'
import { getAssociatedTokenAddress, AccountLayout } from '@solana/spl-token'
import {
  initializeMasterEdition,
  mintPnft,
  updateMetadata,
  burnAndWithdraw,
} from '../../clients/generated/umi/src/instructions'
import fs from 'fs'
import * as path from 'path'
import { transactionBuilder } from '@metaplex-foundation/umi'
import { setComputeUnitLimit, setComputeUnitPrice } from '@metaplex-foundation/mpl-toolbox'

let umi: Umi
let payer: KeypairSigner
let updateAuthority: KeypairSigner

const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')
const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))

describe('initializeMasterEdition Instruction', () => {
  let program: Program<LockedSolPnft>
  let masterMint: PublicKey
  let masterState: PublicKey
  let delegateAuthority: PublicKey
  let collectionAuthorityRecord: PublicKey
  let masterMetadata: PublicKey
  let masterEdition: PublicKey
  let authorityToken: PublicKey

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })
    umi.use(mplTokenMetadata())
    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })
    payer = generateSigner(umi)
    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(1))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(2))

    program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
    const metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

    ;[masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])
    ;[masterState] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('master'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('collection_delegate'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('collection_authority'),
      publicKeySerializer().serialize(delegateAuthority),
    ])
    ;[masterMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[masterEdition] = umi.eddsa.findPda(metadataProgramId, [
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
    authorityToken = publicKey(associatedTokenAccount.toString())
  })

  it('should fail to create a master edition with unauthorized update authority', async () => {
    const unauthorizedAuthority = generateSigner(umi)
    try {
      await initializeMasterEdition(umi, {
        payer,
        masterMint,
        masterMetadata,
        masterEdition,
        updateAuthority: unauthorizedAuthority,
        updateAuthorityToken: authorityToken,
        delegateAuthority,
        collectionAuthorityRecord,
      }).sendAndConfirm(umi)
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Invalid update authority')
      } else {
        throw error
      }
    }
  })

  it('should create a master edition, and associated account with balance 1', async () => {
    const ix = initializeMasterEdition(umi, {
      payer,
      masterMint,
      masterMetadata,
      masterEdition,
      updateAuthority,
      updateAuthorityToken: authorityToken,
      delegateAuthority,
      collectionAuthorityRecord,
    })

    await transactionBuilder()
      .add(setComputeUnitLimit(umi, { units: 400_000 }))
      .add(setComputeUnitPrice(umi, { microLamports: 1 }))
      .add(ix)
      .sendAndConfirm(umi)

    const metadataAccount = await umi.rpc.getAccount(masterMetadata)
    if (!metadataAccount.exists) {
      throw new Error('Metadata account does not exist')
    }
    expect(metadataAccount.exists).toBe(true)

    const metadata = await fetchMetadata(umi, masterMetadata)
    expect(metadata.name).toBe('Locked SOL NFT')
    expect(metadata.symbol).toBe('LSOL')
    expect(metadata.uri).toBe('https://api.locked-sol.com/metadata/initial.json')
    expect(metadata.sellerFeeBasisPoints).toBe(0)
    expect(metadata.isMutable).toBe(true)
    expect(metadata.updateAuthority.toString()).toBe(updateAuthority.publicKey.toString())

    const tokenAccountInfo = await umi.rpc.getAccount(authorityToken)
    expect(tokenAccountInfo).toBeTruthy()

    if (!tokenAccountInfo.exists) {
      throw new Error('Token account does not exist')
    }

    const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
    const balance = BigInt(decodedAccount.amount.toString())
    expect(balance).toBe(BigInt(1))

    // Check collection authority record exists and is properly set up
    const collectionAuthorityRecordAccount = await umi.rpc.getAccount(collectionAuthorityRecord)
    expect(collectionAuthorityRecordAccount.exists).toBe(true)

    // Check master state account exists and has correct delegate
    const masterStateAccount = await umi.rpc.getAccount(masterState)
    expect(masterStateAccount.exists).toBe(true)

    if (masterStateAccount.exists) {
      // Master state layout:
      // discriminator (8) + master_mint (32) + total_minted (8) + delegate (32) + record (32)
      const masterStateData = masterStateAccount.data
      const storedDelegate = new web3PublicKey(masterStateData.slice(48, 80))
      const storedRecord = new web3PublicKey(masterStateData.slice(80, 112))

      expect(storedDelegate.toString()).toBe(delegateAuthority.toString())
      expect(storedRecord.toString()).toBe(collectionAuthorityRecord.toString())
    }
  })

  it('should fail when trying to create a master edition twice', async () => {
    ;[0, 1].forEach(async (count) => {
      try {
        await initializeMasterEdition(umi, {
          payer,
          masterMint,
          masterMetadata,
          masterEdition,
          updateAuthority,
          updateAuthorityToken: authorityToken,
          delegateAuthority,
          collectionAuthorityRecord,
        }).sendAndConfirm(umi)
      } catch (error) {
        if (error instanceof SendTransactionError) {
          expect(error.message).toContain('Transaction simulation failed')
          expect(error.logs).toContainEqual(expect.stringContaining('Allocate: account Address'))
          expect(error.logs).toContainEqual(expect.stringContaining('already in use'))
        } else if (count) {
          throw error
        }
      }
    })
  })
})

describe('mintPnft Instruction', () => {
  let program: Program<LockedSolPnft>

  let payer: KeypairSigner
  let mint: KeypairSigner

  let masterMint: PublicKey
  let masterState: PublicKey
  let delegateAuthority: PublicKey
  let collectionAuthorityRecord: PublicKey
  let masterMetadata: PublicKey
  let masterEdition: PublicKey
  let authorityToken: PublicKey
  let collectionMetadata: PublicKey
  let collectionMasterEdition: PublicKey
  let metadata: PublicKey
  let mintMasterEdition: PublicKey
  let tokenAccount: PublicKey

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })
    umi.use(mplTokenMetadata())
    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })
    payer = generateSigner(umi)
    mint = generateSigner(umi)
    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(2))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(2))

    program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
    const metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

    ;[masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])
    ;[masterState] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('master'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('collection_delegate'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('collection_authority'),
      publicKeySerializer().serialize(delegateAuthority),
    ])
    ;[masterMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[masterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('edition'),
    ])
    ;[collectionMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('edition'),
    ])
    ;[metadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
    ])
    ;[mintMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
      Buffer.from('edition'),
    ])
    ;[delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('collection_delegate'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
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
    authorityToken = publicKey(associatedTokenAccount.toString())

    const nftTokenAccount = await getAssociatedTokenAddress(
      new web3PublicKey(mint.publicKey.toString()),
      new web3PublicKey(payer.publicKey.toString()),
    )
    tokenAccount = publicKey(nftTokenAccount.toString())

    const masterEditionAccount = await umi.rpc.getAccount(masterEdition)
    if (!masterEditionAccount.exists) {
      await initializeMasterEdition(umi, {
        payer,
        masterMint,
        masterMetadata,
        masterEdition,
        updateAuthority,
        updateAuthorityToken: authorityToken,
        delegateAuthority,
        collectionAuthorityRecord,
      }).sendAndConfirm(umi)
    }
  })

  it('should successfully mint a pNFT and verify collection', async () => {
    await mintPnft(umi, {
      payer,
      masterState,
      masterMint,
      collectionMetadata,
      collectionMasterEdition,
      metadata,
      masterEdition: mintMasterEdition,
      mint,
      delegateAuthority,
      collectionAuthorityRecord,
      tokenAccount,
    }).sendAndConfirm(umi)

    const tokenAccountInfo = await umi.rpc.getAccount(tokenAccount)
    if (!tokenAccountInfo.exists) {
      throw new Error('Token account does not exist')
    }
    const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
    expect(BigInt(decodedAccount.amount.toString())).toBe(BigInt(1))

    // Verify vault was created
    const [vaultPda] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('vault'),
      publicKeySerializer().serialize(mint.publicKey),
    ])
    const vaultAccount = await umi.rpc.getAccount(vaultPda)
    expect(vaultAccount.exists).toBe(true)

    const nftMetadata = await fetchMetadata(umi, metadata)
    expect(nftMetadata.name).toBe('Locked SOL NFT')
    expect(nftMetadata.symbol).toBe('LSOL')
    expect(nftMetadata.uri).toBe('https://api.locked-sol.com/metadata/initial.json')
    expect(nftMetadata.collection).not.toBeNull()

    const collection = unwrapOption(nftMetadata.collection)
    if (collection) {
      expect(collection.key.toString()).toBe(masterMint.toString())
      expect(collection.verified).toBe(true)
    } else {
      throw new Error('Collection is None')
    }
  })
})

describe('updateMetadata Instruction', () => {
  let program: Program<LockedSolPnft>
  let metadataProgramId: PublicKey

  let umi: Umi
  let payer: KeypairSigner
  let mint: KeypairSigner
  let updateAuthority: KeypairSigner
  let masterState: PublicKey
  let masterMint: PublicKey
  let masterMetadata: PublicKey
  let metadata: PublicKey
  let masterEdition: PublicKey
  let collectionMetadata: PublicKey
  let collectionMasterEdition: PublicKey
  let mintMasterEdition: PublicKey
  let tokenAccount: PublicKey
  let collectionAuthorityRecord: PublicKey
  let delegateAuthority: PublicKey
  let mintAuthority: PublicKey

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })
    umi.use(mplTokenMetadata())
    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })
    payer = generateSigner(umi)
    mint = generateSigner(umi)
    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(2))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(2))

    program = anchor.workspace.LockedSolPnft
    metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

    // Find PDAs
    ;[masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])
    ;[masterState] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('master'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[metadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
    ])
    ;[masterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('edition'),
    ])
    ;[mintMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
      Buffer.from('edition'),
    ])
    ;[collectionMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('edition'),
    ])
    ;[delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('collection_delegate'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('collection_authority'),
      publicKeySerializer().serialize(delegateAuthority),
    ])
    ;[mintAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('mint_authority'),
      publicKeySerializer().serialize(mint.publicKey),
    ])

    const nftTokenAccount = await getAssociatedTokenAddress(
      new web3PublicKey(mint.publicKey.toString()),
      new web3PublicKey(payer.publicKey.toString()),
    )
    tokenAccount = publicKey(nftTokenAccount.toString())

    // Initialize master edition if not exists
    const masterEditionAccount = await umi.rpc.getAccount(masterEdition)
    if (!masterEditionAccount.exists) {
      const authorityToken = publicKey(
        (
          await getAssociatedTokenAddress(
            new web3PublicKey(masterMint.toString()),
            new web3PublicKey(updateAuthority.publicKey.toString()),
            true,
          )
        ).toString(),
      )

      const masterEditionAccount = await umi.rpc.getAccount(masterEdition)
      if (!masterEditionAccount.exists) {
        await initializeMasterEdition(umi, {
          payer,
          masterMint,
          masterMetadata,
          masterEdition,
          updateAuthority,
          updateAuthorityToken: authorityToken,
          delegateAuthority,
          collectionAuthorityRecord,
        }).sendAndConfirm(umi)
      }
    }
    await mintPnft(umi, {
      payer,
      masterState,
      masterMint,
      collectionMetadata,
      collectionMasterEdition,
      metadata,
      masterEdition: mintMasterEdition,
      mint,
      delegateAuthority,
      collectionAuthorityRecord,
      tokenAccount,
    }).sendAndConfirm(umi)
  })

  it('should update metadata URI and name', async () => {
    const newUri = 'https://api.locked-sol.com/metadata/updated.json'
    const newName = 'Updated LSOL NFT'

    await updateMetadata(umi, {
      serverAuthority: updateAuthority,
      masterState,
      metadata,
      mintAuthority,
      mint: mint.publicKey,
      newUri,
      newName: some(newName),
    }).sendAndConfirm(umi)

    const updatedMetadata = await fetchMetadata(umi, metadata)
    expect(updatedMetadata.uri).toBe(newUri)
    expect(updatedMetadata.name).toBe(newName)
    expect(updatedMetadata.updateAuthority.toString()).toBe(mintAuthority.toString())

    const collection = unwrapOption(updatedMetadata.collection)
    expect(collection).toBeTruthy()
    if (collection) {
      expect(collection.verified).toBe(true)
      expect(collection.key.toString()).toBe(masterMint.toString())
    }

    expect(updatedMetadata.symbol).toBe('LSOL')
    expect(updatedMetadata.sellerFeeBasisPoints).toBe(0)
    expect(updatedMetadata.isMutable).toBe(true)
  })

  it('should fail if user tries to update metadata', async () => {
    try {
      await updateMetadata(umi, {
        serverAuthority: mint,
        masterState,
        mintAuthority,
        mint: mint.publicKey,
        newUri: 'https://api.locked-sol.com/metadata/updated.json',
        newName: none(),
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Unauthorized metadata update')
      }
    }
  })

  it('should fail if another unknown signer tries to update metadata', async () => {
    try {
      await updateMetadata(umi, {
        serverAuthority: generateSigner(umi),
        masterState,
        mintAuthority,
        mint: mint.publicKey,
        newUri: 'https://api.locked-sol.com/metadata/updated.json',
        newName: none(),
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Unauthorized metadata update')
      }
    }
  })
})

describe('burnAndWithdraw Instruction', () => {
  let program: Program<LockedSolPnft>
  let umi: Umi
  let metadataProgramId: PublicKey

  let payer: KeypairSigner
  let mint: KeypairSigner
  let updateAuthority: KeypairSigner

  let masterMint: PublicKey
  let masterState: PublicKey
  let metadata: PublicKey
  let tokenAccount: PublicKey
  let editionMarker: PublicKey
  let delegateAuthority: PublicKey
  let collectionAuthorityRecord: PublicKey
  let masterMetadata: PublicKey
  let masterEdition: PublicKey
  let collectionMetadata: PublicKey
  let collectionMasterEdition: PublicKey
  let mintMasterEdition: PublicKey
  let vault: PublicKey
  let tokenRecord: PublicKey

  beforeEach(async () => {
    // Initialize UMI and program
    umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })
    umi.use(mplTokenMetadata())

    // Set up signers
    const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')
    const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
    const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))

    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })
    payer = generateSigner(umi)
    mint = generateSigner(umi)

    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(2))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(2))

    // Get program and metadata program ID
    program = anchor.workspace.LockedSolPnft
    metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

    // Find all PDAs
    ;[masterMint] = umi.eddsa.findPda(publicKey(program.programId), [Buffer.from('master_mint')])
    ;[masterState] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('master'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[metadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
    ])
    ;[editionMarker] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
      Buffer.from('edition'),
    ])
    ;[masterMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[masterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('edition'),
    ])
    ;[collectionMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('edition'),
    ])
    ;[mintMasterEdition] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
      Buffer.from('edition'),
    ])
    ;[delegateAuthority] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('collection_delegate'),
      publicKeySerializer().serialize(masterMint),
    ])
    ;[collectionAuthorityRecord] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
      Buffer.from('collection_authority'),
      publicKeySerializer().serialize(delegateAuthority),
    ])
    // Get token account
    const nftTokenAccount = await getAssociatedTokenAddress(
      new web3PublicKey(mint.publicKey.toString()),
      new web3PublicKey(payer.publicKey.toString()),
    )
    tokenAccount = publicKey(nftTokenAccount.toString())
    ;[vault] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('vault'),
      publicKeySerializer().serialize(mint.publicKey),
    ])
    ;[tokenRecord] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(mint.publicKey),
      Buffer.from('token_record'),
      publicKeySerializer().serialize(tokenAccount),
    ])

    // Initialize master edition if it doesn't exist
    const masterEditionAccount = await umi.rpc.getAccount(masterEdition)
    if (!masterEditionAccount.exists) {
      const authorityToken = publicKey(
        (
          await getAssociatedTokenAddress(
            new web3PublicKey(masterMint.toString()),
            new web3PublicKey(updateAuthority.publicKey.toString()),
            true,
          )
        ).toString(),
      )

      await initializeMasterEdition(umi, {
        payer,
        masterMint,
        masterMetadata,
        masterEdition,
        updateAuthority,
        updateAuthorityToken: authorityToken,
        delegateAuthority,
        collectionAuthorityRecord,
      }).sendAndConfirm(umi)
    }

    // Mint the NFT that we'll burn
    await mintPnft(umi, {
      payer,
      masterState,
      masterMint,
      collectionMetadata,
      collectionMasterEdition,
      metadata,
      masterEdition: mintMasterEdition,
      mint,
      delegateAuthority,
      collectionAuthorityRecord,
      tokenAccount,
    }).sendAndConfirm(umi)
  })

  it('should successfully burn NFT and withdraw SOL', async () => {
    const sysvarInstructions = publicKey('Sysvar1nstructions1111111111111111111111111')
    console.log('Using sysvar instructions account:', sysvarInstructions.toString())

    console.log('Mint pubkey:', mint.publicKey.toString())
    console.log('Expected vault seeds: vault +', mint.publicKey.toString())
    console.log('Actual vault:', vault.toString())

    const initialVaultBalance = await umi.rpc.getBalance(vault)
    expect(initialVaultBalance.basisPoints).toBeGreaterThan(BigInt(1000000000))

    await transactionBuilder()
      .add(setComputeUnitLimit(umi, { units: 400_000 }))
      .add(
        burnAndWithdraw(umi, {
          owner: payer,
          masterState,
          vault,
          metadata,
          tokenAccount,
          mint: mint.publicKey,
          editionMarker,
          sysvarInstructions,
          collectionMetadata,
          tokenRecord,
        }),
      )
      .sendAndConfirm(umi)

    // Verify token account is closed
    const tokenAccountInfo = await umi.rpc.getAccount(tokenAccount)
    expect(tokenAccountInfo.exists).toBe(false)

    // Verify vault is closed and SOL is transferred
    const vaultAccountInfo = await umi.rpc.getAccount(vault)
    expect(vaultAccountInfo.exists).toBe(false)

    // Verify metadata is burned
    const metadataAccount = await umi.rpc.getAccount(metadata)
    expect(metadataAccount.exists).toBe(false)

    const finalPayerBalance = await umi.rpc.getBalance(payer.publicKey)
    const initialPayerBalance = await umi.rpc.getBalance(payer.publicKey)

    const balanceDifference = finalPayerBalance.basisPoints - initialPayerBalance.basisPoints
    expect(balanceDifference).toBeGreaterThan(BigInt(990000000))
    expect(balanceDifference).toBeLessThan(BigInt(1000000000))
  })
})
