import { Umi, publicKey } from '@metaplex-foundation/umi'
import { AssetV1, fetchAssetsByOwner } from '@metaplex-foundation/mpl-core'
import { COLLECTION_ID } from './consts'

export interface AssetWithImage extends AssetV1 {
  imageUri?: string
}

export async function fetchAssetsByOwnerWithImage(umi: Umi, walletPublicKey: string): Promise<AssetWithImage[]> {
  const ownerAssets = await fetchAssetsByOwner(umi, publicKey(walletPublicKey))

  const collectionAssets = ownerAssets.filter(
    (asset: AssetV1) => asset.updateAuthority.type === 'Collection' && asset.updateAuthority.address === COLLECTION_ID,
  )

  return await Promise.all(
    collectionAssets.map(async (asset: AssetV1) => {
      try {
        if (asset.uri) {
          const response = await fetch(asset.uri)
          const metadata = await response.json()
          return {
            ...asset,
            imageUri: metadata.image,
          }
        }
        return asset
      } catch (error) {
        console.error('Error fetching asset metadata:', error)
        return asset
      }
    }),
  )
}
