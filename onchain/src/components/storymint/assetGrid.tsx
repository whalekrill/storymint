import { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { Umi, publicKey, PublicKey }  from '@metaplex-foundation/umi'
import { burn } from './instructions';
import { AssetWithImage, fetchAssetsByOwnerWithImage } from './utils';

interface AssetGridProps {
  umi: Umi;
  shouldReloadAssets: boolean;
  setShouldReloadAssets: (shouldReload: boolean) => void;
}

const AssetGrid = ({ umi, shouldReloadAssets, setShouldReloadAssets }: AssetGridProps) => {
  const wallet = useWallet();

  const [isLoading, setIsLoading] = useState(true);
  const [assets, setAssets] = useState<AssetWithImage[]>([]);

  const handleBurn = async (assetId: PublicKey) => {
    if (wallet.connected && wallet.publicKey) {
      await burn(umi, assetId);
      setShouldReloadAssets(true);
    }
  }

  useEffect(() => {
    const fetchAssets = async () => {
      if (!wallet.publicKey) return;

      try {
        setIsLoading(true);
        setAssets([]);
        const assets = await fetchAssetsByOwnerWithImage(umi, publicKey(wallet.publicKey));
        setAssets(assets);
      } catch (error) {
        console.error('Error fetching assets:', error);
      } finally {
        setIsLoading(false);
        setShouldReloadAssets(false);
      }
    };

    if (wallet.connected) {
      fetchAssets();
    }
  }, [umi, wallet.connected, wallet.publicKey, shouldReloadAssets, setShouldReloadAssets]);

  if (!wallet.connected) return null;

  if (isLoading) {
    return (
      <div className="flex justify-center items-center p-8">
        <span className="loading loading-spinner loading-lg text-primary"></span>
      </div>
    );
  }

  if (assets.length === 0) {
    return (
      <div className="text-center p-8">
        <p className="text-base-content/60">No assets found in this collection</p>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 p-4">
      {assets.map((asset) => (
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
                onClick={() => handleBurn(asset.publicKey)}
                className="btn btn-error"
              >
                Dissolve Asset
              </button>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

export default AssetGrid;
