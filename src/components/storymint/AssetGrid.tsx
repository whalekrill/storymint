import { PublicKey } from '@metaplex-foundation/umi';
import { AssetWithImage } from './utils';

interface AssetGridProps {
  assets: AssetWithImage[];
  onBurn: (assetId: PublicKey) => Promise<void>;
  burningAsset: string | null;
}

export default function AssetGrid({ assets, onBurn, burningAsset }: AssetGridProps) {
  if (assets.length === 0) {
    return (
      <div className="text-center p-8">
        <p className="text-base-content/60">No assets found in this collection</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-4">
      {assets.map((asset) => {
        const isThisAssetDissolving = burningAsset === asset.publicKey.toString();

        return (
          <div key={asset.publicKey} className="card bg-base-100 shadow-xl">
            <figure className="px-4 pt-4">
              <img
                src={asset.imageUri}
                alt={asset.name}
                className="rounded-xl object-cover w-full aspect-square"
              />
            </figure>
            <div className="card-body items-center text-center">
              <h3 className="card-title">{asset.name}</h3>
              <div className="card-actions">
                <button
                  onClick={() => onBurn(asset.publicKey)}
                  className={`btn btn-error ${isThisAssetDissolving ? 'opacity-70' : ''}`}
                  disabled={isThisAssetDissolving}
                >
                  {isThisAssetDissolving ? 'Dissolving...' : 'Dissolve NFT'}
                </button>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
