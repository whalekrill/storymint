import bs58 from 'bs58'
import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { publicKey, PublicKey, generateSigner, Umi } from '@metaplex-foundation/umi'
import { COLLECTION_ID } from './consts'
import umiModules from './umiModules'

export async function mint(umi: Umi, walletId: PublicKey) {
  const { mintAsset } = await umiModules()
  const programId = umi.programs.get('storymint').publicKey

  const asset = generateSigner(umi)

  const mintAuthority = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('mint_authority'),
    publicKeySerializer().serialize(COLLECTION_ID),
  ])

  const result = await mintAsset(umi, {
    payer: umi.identity,
    collection: COLLECTION_ID,
    owner: publicKey(walletId),
    asset,
    mintAuthority,
  }).sendAndConfirm(umi)

  return bs58.encode(result.signature)
}

export async function burn(umi: Umi, assetId: PublicKey) {
  const { burnAndWithdraw } = await umiModules()
  const programId = umi.programs.get('storymint').publicKey

  const [vault] = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('vault'),
    publicKeySerializer().serialize(assetId),
  ])

  const result = await burnAndWithdraw(umi, {
    owner: umi.identity,
    collection: COLLECTION_ID,
    asset: assetId,
    vault,
  }).sendAndConfirm(umi)

  return bs58.encode(result.signature)
}
