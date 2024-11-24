// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.
import * as fs from 'fs'
import * as path from 'path'
import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { Storymint } from '../target/types/storymint'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { createSignerFromKeypair, keypairIdentity, generateSigner, publicKey } from '@metaplex-foundation/umi'
import { mplCore } from '@metaplex-foundation/mpl-core'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { Keypair } from '@solana/web3.js'
import { initializeCollection } from '../../clients/generated/umi/src/instructions'
import { MPL_TOKEN_METADATA_PROGRAM_ID } from '@metaplex-foundation/mpl-token-metadata'

module.exports = async function (provider: anchor.Provider) {
  const program = anchor.workspace.Storymint as Program<Storymint>
  const programId = program.programId

  const umi = createUmi(provider.connection.rpcEndpoint, {
    commitment: provider.connection.commitment,
  })
  umi.use(mplCore())

  const keypairPath = path.join(__dirname, '../keys/update-authority-devnet.json')
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
  const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))

  const updateAuthority = createSignerFromKeypair(umi, {
    publicKey: publicKey(keyPair.publicKey.toString()),
    secretKey: keyPair.secretKey,
  })
  umi.use(keypairIdentity(updateAuthority))

  const collection = generateSigner(umi)

  console.log('Initializing master edition...')

  // Find PDA for mint authority
  const mintAuthority = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('mint_authority'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  // Get metadata account address
  const [collectionMetadata] = umi.eddsa.findPda(publicKey(MPL_TOKEN_METADATA_PROGRAM_ID), [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  // Get collection authority record PDA
  const [collectionAuthorityRecord] = umi.eddsa.findPda(publicKey(MPL_TOKEN_METADATA_PROGRAM_ID), [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(collection.publicKey),
    Buffer.from('collection_authority'),
    publicKeySerializer().serialize(mintAuthority),
  ])

  const initializeCollectionArgs = {
    collectionMetadata,
    collectionAuthorityRecord,
    mintAuthority,
    args: {
      name: 'Storymint',
      uri: 'https://storage.googleapis.com/storymint/metadata/d251b52f-51a4-46bd-ad0a-2e3eca0c90cb.json',
    },
  }

  await initializeCollection(umi, {
    payer: updateAuthority,
    collection,
    updateAuthority,
    ...initializeCollectionArgs,
  }).sendAndConfirm(umi)

  console.log(`Done initializing collection ${collection.publicKey.toString()}`)
}
