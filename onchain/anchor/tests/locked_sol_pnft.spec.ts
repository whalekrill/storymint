import * as anchor from '@coral-xyz/anchor'
import { Program } from '@coral-xyz/anchor'
import { LockedSolPnft } from '../target/types/locked_sol_pnft'
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { createSignerFromKeypair, keypairIdentity, sol, publicKey, generateSigner } from '@metaplex-foundation/umi'
import { mplTokenMetadata } from '@metaplex-foundation/mpl-token-metadata'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { Keypair } from '@solana/web3.js'
import { PublicKey as web3PublicKey } from '@solana/web3.js'
import { getAssociatedTokenAddress, AccountLayout } from '@solana/spl-token'
import { initializeMasterEdition } from '../../clients/generated/umi/src/instructions'
import fs from 'fs'
import * as path from 'path'

describe('initializeMasterEdition Instruction', () => {
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

  const program = anchor.workspace.LockedSolPnft as Program<LockedSolPnft>
  const metadataProgramId = umi.programs.getPublicKey('mplTokenMetadata')

  it('should create a master edition, and associated account with balance 1', async () => {
    await umi.rpc.airdrop(payer.publicKey, sol(1))

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
    expect(tokenAccountInfo).toBeTruthy()

    if (!tokenAccountInfo.exists) {
      throw new Error('Token account does not exist')
    }

    const decodedAccount = AccountLayout.decode(tokenAccountInfo.data)
    const balance = BigInt(decodedAccount.amount.toString())
    expect(balance).toBe(BigInt(1))
  })
})
