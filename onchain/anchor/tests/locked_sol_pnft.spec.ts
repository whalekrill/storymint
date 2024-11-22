import fs from 'fs'
import * as path from 'path'
import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import {
  Umi,
  PublicKey,
  KeypairSigner,
  Signer,
  createSignerFromKeypair,
  keypairIdentity,
  sol,
  publicKey,
  generateSigner,
} from '@metaplex-foundation/umi'
import { mplCore, fetchAssetV1 } from '@metaplex-foundation/mpl-core'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { SendTransactionError, Keypair } from '@solana/web3.js'
import { transactionBuilder } from '@metaplex-foundation/umi'
import { setComputeUnitLimit } from '@metaplex-foundation/mpl-toolbox'
import {
  initializeCollection,
  mintAsset,
  updateMetadata,
  burnAndWithdraw,
} from '../../clients/generated/umi/src/instructions'

const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')
const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))

describe('Initialize Collection', () => {
  let umi: Umi
  let program: Program<LockedSolPnft>
  let payer: KeypairSigner
  let collection: PublicKey
  let updateAuthority: KeypairSigner

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899').use(mplCore())
    program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>

    payer = generateSigner(umi)
    const collectionSigner = generateSigner(umi)
    collection = collectionSigner.publicKey

    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })

    umi.use(keypairIdentity(payer))

    await umi.rpc.airdrop(payer.publicKey, sol(1))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(1))
  })

  it('should initialize collection with correct metadata', async () => {
    await initializeCollection(umi, {
      payer,
      collection,
      updateAuthority,
      name: 'Locked SOL NFT',
      uri: 'https://api.locked-sol.com/metadata/initial.json',
    }).sendAndConfirm(umi)

    const assetData = await fetchAssetV1(umi, collection)
    expect(assetData.name).toBe('Locked SOL NFT')
    expect(assetData.uri).toBe('https://api.locked-sol.com/metadata/initial.json')
    expect(assetData.updateAuthority.toString()).toBe(updateAuthority.publicKey.toString())

    const masterStateSeeds = [Buffer.from('master'), publicKeySerializer().serialize(collection)]
    const [masterState] = umi.eddsa.findPda(publicKey(program.programId), masterStateSeeds)
    const masterStateAccount = await umi.rpc.getAccount(masterState)
    expect(masterStateAccount.exists).toBe(true)
  })

  it('should fail with unauthorized update authority', async () => {
    const wrongAuthority = generateSigner(umi)

    try {
      await initializeCollection(umi, {
        payer,
        collection,
        updateAuthority: wrongAuthority,
        name: 'Locked SOL NFT',
        uri: 'https://api.locked-sol.com/metadata/initial.json',
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Invalid update authority')
      }
    }
  })
})

describe('Mint Asset', () => {
  let umi: Umi
  let program: Program<LockedSolPnft>
  let payer: KeypairSigner
  let asset: KeypairSigner
  let collection: PublicKey
  let updateAuthority: KeypairSigner

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899').use(mplCore())
    program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>

    payer = generateSigner(umi)
    asset = generateSigner(umi)
    const collectionSigner = generateSigner(umi)
    collection = collectionSigner.publicKey

    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })

    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(2))
  })

  it('should successfully mint an asset', async () => {
    await transactionBuilder()
      .add(setComputeUnitLimit(umi, { units: 400_000 }))
      .add(
        mintAsset(umi, {
          payer,
          asset,
          collection,
          updateAuthority: updateAuthority.publicKey,
          args: {
            name: 'Locked SOL NFT',
            uri: 'https://api.locked-sol.com/metadata/initial.json',
            plugins: null,
          },
        }),
      )
      .sendAndConfirm(umi)

    const assetData = await fetchAssetV1(umi, asset.publicKey)
    expect(assetData.name).toBe('Locked SOL NFT')
    expect(assetData.uri).toBe('https://api.locked-sol.com/metadata/initial.json')
    expect(assetData.updateAuthority.toString()).toBe(updateAuthority.publicKey.toString())

    const vaultSeeds = [Buffer.from('vault'), publicKeySerializer().serialize(asset)]
    const [vault] = umi.eddsa.findPda(publicKey(program.programId), vaultSeeds)
    const vaultAccount = await umi.rpc.getAccount(vault)
    expect(vaultAccount.exists).toBe(true)
  })

  it('should update master state total minted', async () => {
    const [masterState] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('master'),
      publicKeySerializer().serialize(collection),
    ])

    const beforeState = await umi.rpc.getAccount(masterState)
    const beforeMinted = beforeState.exists ? Number(beforeState.data.slice(40, 48)) : 0

    await mintAsset(umi, {
      payer,
      asset,
      collection,
      updateAuthority: updateAuthority.publicKey,
      args: {
        name: 'Locked SOL NFT',
        uri: 'https://api.locked-sol.com/metadata/initial.json',
        plugins: null,
      },
    }).sendAndConfirm(umi)

    const mState = await umi.rpc.getAccount(masterState)
    expect(mState.exists).toBe(true)
    if (mState.exists) {
      const afterMinted = Number(mState.data.slice(40, 48))
      expect(afterMinted).toBe(beforeMinted + 1)
    }
  })
})

describe('Update Metadata', () => {
  let umi: Umi
  let payer: KeypairSigner
  let asset: KeypairSigner
  let updateAuthority: KeypairSigner
  let collection: PublicKey

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899').use(mplCore())

    payer = generateSigner(umi)
    asset = generateSigner(umi)
    const collectionSigner = generateSigner(umi)
    collection = collectionSigner.publicKey

    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })

    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(1))
    await umi.rpc.airdrop(updateAuthority.publicKey, sol(1))

    await mintAsset(umi, {
      payer,
      asset,
      collection,
      updateAuthority: updateAuthority.publicKey,
      args: {
        name: 'Locked SOL NFT',
        uri: 'https://api.locked-sol.com/metadata/initial.json',
        plugins: null,
      },
    }).sendAndConfirm(umi)
  })

  it('should update metadata URI and name', async () => {
    const newUri = 'https://api.locked-sol.com/metadata/updated.json'
    const newName = 'Updated LSOL NFT'

    await updateMetadata(umi, {
      asset: asset.publicKey,
      collection,
      authority: updateAuthority,
      payer,
      args: {
        name: newName,
        uri: newUri,
      },
    }).sendAndConfirm(umi)

    const assetData = await fetchAssetV1(umi, asset.publicKey)
    expect(assetData.name).toBe(newName)
    expect(assetData.uri).toBe(newUri)
  })

  it('should fail with unauthorized authority', async () => {
    const wrongAuthority = generateSigner(umi)

    try {
      await updateMetadata(umi, {
        asset: asset.publicKey,
        collection,
        authority: wrongAuthority,
        payer,
        args: {
          name: 'Updated Name',
          uri: 'https://api.locked-sol.com/metadata/updated.json',
        },
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Unauthorized metadata update')
      }
    }
  })
})

describe('Burn and Withdraw', () => {
  let umi: Umi
  let program: Program<LockedSolPnft>
  let payer: KeypairSigner
  let asset: Signer
  let updateAuthority: KeypairSigner
  let collection: PublicKey

  beforeEach(async () => {
    umi = createUmi('http://127.0.0.1:8899').use(mplCore())
    program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>

    payer = generateSigner(umi)
    asset = generateSigner(umi)
    const collectionSigner = generateSigner(umi)
    collection = collectionSigner.publicKey

    updateAuthority = createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })

    umi.use(keypairIdentity(payer))
    await umi.rpc.airdrop(payer.publicKey, sol(2))

    await mintAsset(umi, {
      payer,
      asset,
      collection,
      updateAuthority: updateAuthority.publicKey,
      args: {
        name: 'Locked SOL NFT',
        uri: 'https://api.locked-sol.com/metadata/initial.json',
        plugins: null,
      },
    }).sendAndConfirm(umi)
  })

  it('should burn asset and withdraw SOL', async () => {
    const [vault] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('vault'),
      publicKeySerializer().serialize(asset),
    ])

    const initialVaultBalance = await umi.rpc.getBalance(vault)
    expect(initialVaultBalance.basisPoints).toBe(BigInt(1000000000))

    const initialPayerBalance = await umi.rpc.getBalance(payer.publicKey)

    await burnAndWithdraw(umi, {
      owner: payer,
      asset: asset.publicKey,
      collection,
      vault,
    }).sendAndConfirm(umi)

    const assetAccount = await umi.rpc.getAccount(asset.publicKey)
    expect(assetAccount.exists).toBe(false)

    const vaultAccount = await umi.rpc.getAccount(vault)
    expect(vaultAccount.exists).toBe(false)

    const finalPayerBalance = await umi.rpc.getBalance(payer.publicKey)
    const balanceDifference = finalPayerBalance.basisPoints - initialPayerBalance.basisPoints
    expect(balanceDifference).toBeGreaterThan(BigInt(990000000))
    expect(balanceDifference).toBeLessThan(BigInt(1000000000))
  })

  it('should fail with wrong owner', async () => {
    const wrongOwner = generateSigner(umi)

    try {
      await burnAndWithdraw(umi, {
        owner: wrongOwner,
        asset: asset.publicKey,
        collection,
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('unauthorized')
      }
    }
  })
})
