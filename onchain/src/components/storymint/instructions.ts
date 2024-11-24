import { publicKey as publicKeySerializer } from '@metaplex-foundation/umi/serializers'
import { mintAsset, burnAndWithdraw } from '../../../clients/generated/umi/src/'
import { publicKey, PublicKey, generateSigner, Umi } from '@metaplex-foundation/umi'
import { COLLECTION_ID } from './consts'

export async function mint(umi: Umi, walletId: PublicKey) {
  const programId = umi.programs.get('storymint').publicKey

  const asset = generateSigner(umi)

  const mintAuthority = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('mint_authority'),
    publicKeySerializer().serialize(COLLECTION_ID),
  ])

  await mintAsset(umi, {
    payer: umi.identity,
    collection: COLLECTION_ID,
    owner: publicKey(walletId),
    asset,
    mintAuthority,
  }).sendAndConfirm(umi)
}

export async function burn(umi: Umi, assetId: PublicKey) {
  const programId = umi.programs.get('storymint').publicKey

  const [vault] = umi.eddsa.findPda(publicKey(programId), [
    Buffer.from('vault'),
    publicKeySerializer().serialize(assetId),
  ])

  await burnAndWithdraw(umi, {
    owner: umi.identity,
    collection: COLLECTION_ID,
    asset: assetId,
    vault,
  }).sendAndConfirm(umi)
}
