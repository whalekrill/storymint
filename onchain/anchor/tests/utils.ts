import fs from 'fs'
import * as path from 'path'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { COLLECTION_NAME, COLLECTION_URI } from './consts'
import {
  Umi,
  publicKey,
  generateSigner,
  PublicKey,
  KeypairSigner,
  createSignerFromKeypair,
} from '@metaplex-foundation/umi'
import { Keypair } from '@solana/web3.js'

export function getUpdateAuthority(umi: Umi) {
  const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
  const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))
  return createSignerFromKeypair(umi, {
    publicKey: publicKey(keyPair.publicKey.toString()),
    secretKey: keyPair.secretKey,
  })
}

export function initializeCollectionArgs(umi: Umi, programId: PublicKey, collection: KeypairSigner) {
  // Find PDA for mint authority
  const mintAuthority = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('mint_authority'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  // Get metadata account address
  const [collectionMetadata] = umi.eddsa.findPda(publicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'), [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  // Get collection authority record PDA
  const [collectionAuthorityRecord] = umi.eddsa.findPda(publicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'), [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(collection.publicKey),
    Buffer.from('collection_authority'),
    publicKeySerializer().serialize(mintAuthority),
  ])

  return {
    collectionMetadata,
    collectionAuthorityRecord,
    mintAuthority,
    args: { name: COLLECTION_NAME, uri: COLLECTION_URI },
  }
}

export function mintAssetArgs(umi: Umi, programId: PublicKey, collection: KeypairSigner) {
  const asset = generateSigner(umi)

  const mintAuthority = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('mint_authority'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  return { asset, mintAuthority }
}

export function burnAndWithdrawArgs(umi: Umi, programId: PublicKey, collection: KeypairSigner) {
  const { asset, mintAuthority } = mintAssetArgs(umi, programId, collection)

  const [vault] = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('vault'),
    publicKeySerializer().serialize(asset),
  ])

  return { asset, mintAuthority, vault }
}
