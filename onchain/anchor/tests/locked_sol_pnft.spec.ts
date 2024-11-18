import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { createNoopSigner, keypairIdentity, sol, publicKey, generateSigner } from '@metaplex-foundation/umi'
import { string, publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { initializeMasterEdition } from '../../clients/generated/umi/src/instructions'
import fs from 'fs'
import * as path from 'path'

const umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })

// Define the path to your keypair JSON file
const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')

// Read and parse the JSON file
const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))

// Convert the secret key array to a Uint8Array
const secretKeyUint8Array = Uint8Array.from(secretKey)

// Use `generateSigner` to create a valid signer from the secret key
const signer = generateSigner(secretKeyUint8Array)

// Use the public key from the keypair
const updateAuthority = createNoopSigner(signer.publicKey)

describe('initializeMasterEdition Instruction', () => {
  const payer = generateSigner(umi)
  umi.use(keypairIdentity(payer))

  const mint = generateSigner(umi).publicKey

  const tokenMetadataProgramId = publicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s')
  const masterMetadata = umi.eddsa.findPda(tokenMetadataProgramId, [
    string({ size: 'variable' }).serialize('metadata'),
    publicKeySerializer().serialize(mint),
  ])

  it('should create a master edition successfully', async () => {
    // Airdrop SOL to payer for transaction fees
    await umi.rpc.airdrop(payer.publicKey, sol(1), { commitment: 'processed' })

    const tx = initializeMasterEdition(umi, {
      payer,
      masterMetadata,
      masterEdition: umi.eddsa.findPda(payer.publicKey, [Buffer.from('master_edition')]),
      updateAuthority,
      authorityToken: umi.eddsa.findPda(payer.publicKey, [Buffer.from('authority')]),
    }).sendAndConfirm(umi)

    expect(tx).toBeDefined()
  })
})
