import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { createSignerFromKeypair, keypairIdentity, sol, publicKey, generateSigner } from '@metaplex-foundation/umi'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { Keypair } from '@solana/web3.js'
import { PublicKey as web3PublicKey } from '@solana/web3.js'
import { getAssociatedTokenAddress, AccountLayout } from '@solana/spl-token'
import { initializeMasterEdition } from '../../clients/generated/umi/src/instructions'
import fs from 'fs'
import * as path from 'path'

describe('initializeMasterEdition Instruction', () => {
  const umi = createUmi('http://127.0.0.1:8899', { commitment: 'processed' })

  const keypairPath = path.join(__dirname, '../../../keys/update-authority-devnet.json')
  const secretKey = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))
  const keyPair = Keypair.fromSecretKey(Uint8Array.from(secretKey))
  const updateAuthority = createSignerFromKeypair(umi, {
    publicKey: publicKey(keyPair.publicKey.toString()),
    secretKey: keyPair.secretKey,
  })

  const payer = generateSigner(umi)
  umi.use(keypairIdentity(payer))

  const programId = publicKey('3kLyy6249ZFsZyG74b6eSwuvDUVndkFM54cvK8gnietr')
  const metadataProgramId = publicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s')

  it('should create a master edition successfully', async () => {
    await umi.rpc.airdrop(payer.publicKey, sol(1))

    // Debug the seed being used
    const masterMintSeed = 'master_mint'
    console.log('Master Mint Seed:', masterMintSeed)
    console.log('Master Mint Seed Bytes:', Array.from(Buffer.from(masterMintSeed)))

    // Derive master mint PDA
    const [masterMint] = umi.eddsa.findPda(programId, [Buffer.from(masterMintSeed)])

    console.log('\nDerived PDAs:')
    console.log('Master Mint:', masterMint.toString())

    // Derive metadata PDA
    const [masterMetadata] = umi.eddsa.findPda(metadataProgramId, [
      Buffer.from('metadata'),
      publicKeySerializer().serialize(metadataProgramId),
      publicKeySerializer().serialize(masterMint),
    ])

    // Derive master edition PDA
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

    console.log('\nAccount Details:')
    console.log('Program ID:', programId.toString())
    console.log('Payer:', payer.publicKey.toString())
    console.log('Master Mint:', masterMint.toString())
    console.log('Master Metadata:', masterMetadata.toString())
    console.log('Master Edition:', masterEdition.toString())
    console.log('Update Authority:', updateAuthority.publicKey.toString())
    console.log('Authority Token:', authorityToken.toString())

    try {
      await initializeMasterEdition(umi, {
        payer,
        masterMint,
        masterMetadata,
        masterEdition,
        updateAuthority,
        authorityToken,
      }).sendAndConfirm(umi)

      console.log('Successfully initialized master edition')

      const tokenAccountInfo = await umi.rpc.getAccount(publicKey(associatedTokenAccount.toString()))
      expect(tokenAccountInfo).toBeTruthy()
      console.log('Token Account Info:', tokenAccountInfo)

      if (!tokenAccountInfo.exists) {
        throw new Error('Token account does not exist')
      }

      const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
      const balance = BigInt(decodedAccount.amount.toString()) // Convert balance to BigInt

      console.log('Decoded Token Balance (web3.js):', balance.toString())
      expect(balance).toBe(BigInt(1))
    } catch (error) {
      if (error instanceof Error) {
        console.error('\nError Details:')
        // @ts-ignore
        if (typeof error === 'object' && error !== null && 'getLogs' in error && typeof error.getLogs === 'function') {
          // @ts-ignore
          const logs = await error.getLogs()
          console.log('\nTransaction logs:', logs)
        }
      }
      throw error
    }
  })
})
