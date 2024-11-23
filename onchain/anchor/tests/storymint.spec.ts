import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { mplCore, fetchAssetV1 } from '@metaplex-foundation/mpl-core'
import { keypairIdentity, publicKey, sol, generateSigner } from '@metaplex-foundation/umi'
import { SendTransactionError } from '@solana/web3.js'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import {
  burnAndWithdraw,
  initializeCollection,
  mintAsset,
  updateMetadata,
} from '../../clients/generated/umi/src/instructions'
import { getUpdateAuthority, initializeCollectionArgs, mintAssetArgs, burnAndWithdrawArgs } from './utils'

jest.setTimeout(100000)

describe('Storymint', () => {
  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
  const umi = createUmi('http://127.0.0.1:8899').use(mplCore())

  const payer = generateSigner(umi)
  umi.use(keypairIdentity(payer))
  const updateAuthority = getUpdateAuthority(umi)

  const collection = generateSigner(umi)

  beforeAll(async () => {
    await umi.rpc.airdrop(payer.publicKey, sol(100))
    await initializeCollection(umi, {
      payer,
      collection,
      updateAuthority,
      ...initializeCollectionArgs(umi, publicKey(program.programId), collection),
    }).sendAndConfirm(umi)
  })

  it('should initialize collection with correct master state', async () => {
    const [masterStateAddress] = umi.eddsa.findPda(publicKey(program.programId), [
      Buffer.from('master'),
      publicKeySerializer().serialize(collection),
    ])

    const masterStateAccount = await umi.rpc.getAccount(masterStateAddress)
    if (!masterStateAccount.exists) {
      throw new Error('Master state account not found')
    }
    const data = masterStateAccount.data

    // Verify the collection pubkey matches
    expect(publicKey(Buffer.from(data.slice(8, 40)))).toEqual(collection.publicKey)

    // Verify total_minted starts at 0
    const totalMinted = data.slice(40, 48)
    expect(Buffer.from(totalMinted).readBigUInt64LE()).toBe(BigInt(0))

    // Verify account size
    expect(data.length).toBe(48)
  })

  it('should fail with unauthorized update authority', async () => {
    const collection = generateSigner(umi)
    try {
      await initializeCollection(umi, {
        payer,
        collection,
        updateAuthority: generateSigner(umi),
        ...initializeCollectionArgs(umi, publicKey(program.programId), collection),
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('InvalidUpdateAuthority')
      }
    }
  })

  it('should fail to initialize same collection twice', async () => {
    const collection = generateSigner(umi)
    try {
      await initializeCollection(umi, {
        payer,
        collection,
        updateAuthority,
        ...initializeCollectionArgs(umi, publicKey(program.programId), collection),
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('AlreadyInitialized')
      }
    }
  })

  it('should successfully mint an asset', async () => {
    const { asset, mintAuthority } = mintAssetArgs(umi, publicKey(program.programId), collection)

    await mintAsset(umi, {
      payer,
      collection: collection.publicKey,
      owner: payer.publicKey,
      asset,
      mintAuthority,
    }).sendAndConfirm(umi)

    const assetData = await fetchAssetV1(umi, asset.publicKey)
    expect(assetData.name).toBe('Locked SOL NFT')
    expect(assetData.uri).toBe('https://api.locked-sol.com/metadata/initial.json')

    const vaultSeeds = [Buffer.from('vault'), publicKeySerializer().serialize(asset)]
    const [vault] = umi.eddsa.findPda(publicKey(program.programId), vaultSeeds)

    const vaultAccount = await umi.rpc.getAccount(vault)
    expect(vaultAccount.exists).toBe(true)

    const vaultBalance = await umi.rpc.getBalance(vault)
    expect(vaultBalance.basisPoints).toBeGreaterThan(BigInt(1000000000))
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
      collection: collection.publicKey,
      owner: payer.publicKey,
      ...mintAssetArgs(umi, publicKey(program.programId), collection),
    }).sendAndConfirm(umi)

    const newMasterState = await umi.rpc.getAccount(masterState)
    expect(newMasterState.exists).toBe(true)
    if (newMasterState.exists) {
      const afterMinted = Number(newMasterState.data.slice(40, 48))
      expect(afterMinted).toBe(beforeMinted + 1)
    }
  })

  it('should update metadata URI and name', async () => {
    const { asset, mintAuthority } = mintAssetArgs(umi, publicKey(program.programId), collection)

    await mintAsset(umi, {
      payer,
      collection: collection.publicKey,
      owner: payer.publicKey,
      asset,
      mintAuthority,
    }).sendAndConfirm(umi)

    const newUri = 'https://api.locked-sol.com/metadata/updated.json'
    const newName = 'Updated LSOL NFT'

    await updateMetadata(umi, {
      asset: asset.publicKey,
      collection: collection.publicKey,
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

    const { asset, mintAuthority } = mintAssetArgs(umi, publicKey(program.programId), collection)

    await mintAsset(umi, {
      payer,
      collection: collection.publicKey,
      owner: payer.publicKey,
      asset,
      mintAuthority,
    }).sendAndConfirm(umi)

    try {
      await updateMetadata(umi, {
        asset: asset.publicKey,
        collection: collection.publicKey,
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

  it('should burn asset and withdraw SOL', async () => {
    const { asset, mintAuthority, vault } = burnAndWithdrawArgs(umi, publicKey(program.programId), collection)

    const initialPayerBalance = await umi.rpc.getBalance(payer.publicKey)

    await mintAsset(umi, {
      payer,
      collection: collection.publicKey,
      owner: payer.publicKey,
      asset,
      mintAuthority,
    }).sendAndConfirm(umi)

    const initialVaultBalance = await umi.rpc.getBalance(vault)
    expect(initialVaultBalance.basisPoints).toBeGreaterThan(BigInt(1000000000))

    await burnAndWithdraw(umi, {
      owner: payer,
      collection: collection.publicKey,
      asset: asset.publicKey,
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

  it('should fail with update authority', async () => {
    const { asset, mintAuthority, vault } = burnAndWithdrawArgs(umi, publicKey(program.programId), collection)

    await mintAsset(umi, {
      payer,
      collection: collection.publicKey,
      owner: payer.publicKey,
      asset,
      mintAuthority,
    }).sendAndConfirm(umi)

    try {
      await burnAndWithdraw(umi, {
        owner: updateAuthority,
        collection: collection.publicKey,
        asset: asset.publicKey,
        vault,
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Neither the asset or any plugins have approved this operation')
      }
    }
  })

  it('should fail with wrong owner', async () => {
    const wrongOwner = generateSigner(umi)

    const { asset, mintAuthority, vault } = burnAndWithdrawArgs(umi, publicKey(program.programId), collection)

    await mintAsset(umi, {
      payer,
      collection: collection.publicKey,
      owner: payer.publicKey,
      asset,
      mintAuthority,
    }).sendAndConfirm(umi)

    try {
      await burnAndWithdraw(umi, {
        owner: wrongOwner,
        collection: collection.publicKey,
        asset: asset.publicKey,
        vault,
      }).sendAndConfirm(umi)
      fail('Should have thrown error')
    } catch (error) {
      if (error instanceof SendTransactionError) {
        expect(error.message).toContain('Neither the asset or any plugins have approved this operation')
      }
    }
  })
})
