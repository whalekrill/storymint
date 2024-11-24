import 'dotenv/config'
import * as fs from 'fs'
import * as path from 'path'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import {
  Umi,
  PublicKey,
  KeypairSigner,
  createSignerFromKeypair,
  keypairIdentity,
  generateSigner,
  publicKey,
} from '@metaplex-foundation/umi'
import { mplCore } from '@metaplex-foundation/mpl-core'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { SendTransactionError, Keypair } from '@solana/web3.js'
import { initializeCollection } from '../clients/generated/umi/src/instructions'
import { createStorymintProgram } from '../clients/generated/umi/src/'

async function getUpdateAuthority(umi: Umi) {
  try {
    const keypairPath = path.join(process.cwd(), '../keys/update-authority-devnet.json')
    const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
    const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))

    return createSignerFromKeypair(umi, {
      publicKey: publicKey(keyPair.publicKey.toString()),
      secretKey: keyPair.secretKey,
    })
  } catch (error) {
    console.error('Failed to load update authority:', error)
    throw error
  }
}

async function getProgramAddresses(umi: Umi, programId: PublicKey, collection: KeypairSigner) {
  const MPL_TOKEN_METADATA_PROGRAM_ID = publicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s')

  const mintAuthority = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('mint_authority'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  const [collectionMetadata] = umi.eddsa.findPda(publicKey(MPL_TOKEN_METADATA_PROGRAM_ID), [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(collection.publicKey),
  ])

  const [collectionAuthorityRecord] = umi.eddsa.findPda(publicKey(MPL_TOKEN_METADATA_PROGRAM_ID), [
    Buffer.from('metadata'),
    publicKeySerializer().serialize(collection.publicKey),
    Buffer.from('collection_authority'),
    publicKeySerializer().serialize(mintAuthority),
  ])

  return {
    mintAuthority,
    collectionMetadata,
    collectionAuthorityRecord,
  }
}

async function initialize() {
  try {
    console.log('Starting collection initialization...')

    const umi = createUmi(process.env.CLUSTER_URL)
      .use(mplCore())
      .use({
        install(umi) {
          umi.programs.add(createStorymintProgram())
        },
      })
    const programId = umi.programs.get('storymint').publicKey

    const updateAuthority = await getUpdateAuthority(umi)
    umi.use(keypairIdentity(updateAuthority))

    const collection = generateSigner(umi)
    console.log('Collection address:', collection.publicKey.toString())

    const { mintAuthority, collectionMetadata, collectionAuthorityRecord } = await getProgramAddresses(
      umi,
      programId,
      collection,
    )

    console.log('PDAs generated:')
    console.log('Mint Authority:', mintAuthority.toString())
    console.log('Collection Metadata:', collectionMetadata.toString())
    console.log('Collection Authority Record:', collectionAuthorityRecord.toString())

    console.log('Initializing collection...')

    const initializeCollectionArgs = {
      collectionMetadata,
      collectionAuthorityRecord,
      mintAuthority,
      args: {
        name: 'Storymint',
        uri: 'https://storage.googleapis.com/storymint/metadata/d251b52f-51a4-46bd-ad0a-2e3eca0c90cb.json',
      },
    }

    const signature = await initializeCollection(umi, {
      payer: updateAuthority,
      collection,
      updateAuthority,
      ...initializeCollectionArgs,
    }).sendAndConfirm(umi)

    console.log('Collection initialized successfully!')
    console.log('Collection address:', collection.publicKey.toString())
    console.log('Transaction signature:', signature)
  } catch (error) {
    if (error instanceof SendTransactionError) {
      console.error('Failed to initialize collection:', error)
      if (error.logs) {
        console.error('Transaction logs:', error.logs)
      }
    }
    throw error
  }
}

initialize().catch((error) => {
  console.error('Initialization failed:', error)
  process.exit(1)
})
