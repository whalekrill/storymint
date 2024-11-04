import { useState, useEffect, useCallback } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults';
import { WalletButton } from '../solana/solana-provider';
import { AppHero } from '../ui/ui-layout';
import { mplCore } from '@metaplex-foundation/mpl-core';
import { Umi, publicKey, PublicKey } from '@metaplex-foundation/umi';
import { walletAdapterIdentity } from '@metaplex-foundation/umi-signer-wallet-adapters';
import { mint, burn } from './instructions';
import { AssetWithImage, fetchAssetsByOwnerWithImage } from './utils';
import AssetGrid  from './AssetGrid';
import { useTransactionToast } from '../ui/ui-layout';
import umiModules from './umiModules';

export default function Storymint() {
  const wallet = useWallet();
  const [umi, setUmi] = useState<Umi | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [assets, setAssets] = useState<AssetWithImage[]>([]);
  const [isMinting, setIsMinting] = useState(false);
  const [burningAsset, setBurningAsset] = useState<string | null>(null);
  const showTransactionToast = useTransactionToast();

  // Initialize Umi
  useEffect(() => {
    const init = async () => {
      const cluster = import.meta.env.VITE_CLUSTER_URL;
      if (!cluster) {
        throw new Error('VITE_CLUSTER_URL is not defined');
      }
      
      const { createStorymintProgram } = await umiModules();
      const newUmi = createUmi(cluster)
        .use(mplCore())
        .use({
          install(umi) {
            umi.programs.add(createStorymintProgram());
          },
        })
        .use(walletAdapterIdentity(wallet));
        
      setUmi(newUmi);
    };
    init();
  }, [wallet]);

  useEffect(() => {
    const initialFetch = async () => {
      if (!wallet.publicKey || !umi) return;
      setIsLoading(true);
      try {
        const newAssets = await fetchAssetsByOwnerWithImage(
          umi,
          publicKey(wallet.publicKey)
        );
        setAssets(newAssets);
      } catch (error) {
        console.error('Error fetching assets:', error);
      } finally {
        setIsLoading(false);
      }
    };

    initialFetch();
  }, [wallet.publicKey, umi]); 

  const fetchAssetsWithRetry = useCallback(async () => {
    if (!wallet.publicKey || !umi) return;
    
    let attempts = 0;
    let lastAssets = assets;
    const delaySequence = [0, 1, 3, 5, 7, 10];
    const maxAttempts = delaySequence.length;

    while (attempts < maxAttempts) {
      try {
        const newAssets = await fetchAssetsByOwnerWithImage(
          umi,
          publicKey(wallet.publicKey)
        );
        if (newAssets.length !== lastAssets.length) {
          setAssets(newAssets);
          return;
        }
        lastAssets = newAssets;
        attempts++;
        if (attempts < maxAttempts) {
          await new Promise(resolve => 
            setTimeout(resolve, delaySequence[attempts] * 1000)
          );
        }
      } catch (error) {
        console.error('Error fetching assets:', error);
        attempts++;
        if (attempts === maxAttempts) break;
        await new Promise(resolve => 
            setTimeout(resolve, delaySequence[attempts] * 1000)
        );
      }
    }
  }, [wallet.publicKey, umi, assets]);

 const handleMint = async () => {
    if (umi && wallet.connected && wallet.publicKey) {
      try {
        setIsMinting(true);
        const result = await mint(umi, publicKey(wallet.publicKey));
        if (result) {
          showTransactionToast(result);
        }
        await fetchAssetsWithRetry();
      } catch (error) {
        console.error('Error minting:', error);
      } finally {
        setIsMinting(false);
      }
    }
  };

  const handleBurn = async (assetId: PublicKey) => {
    if (umi && wallet.connected && wallet.publicKey) {
      try {
        setBurningAsset(assetId.toString());
        const result = await burn(umi, assetId);
        if (result) {
          showTransactionToast(result);
        }
        await fetchAssetsWithRetry();
      } catch (error) {
        console.error('Error burning asset:', error);
      } finally {
        setBurningAsset(null);
      }
    }
  };

  if (!umi) return null;

  if (!wallet.connected) {
    return (
      <div className="max-w-4xl mx-auto">
        <div className="hero py-16">
          <div className="hero-content text-center">
            <div className="flex flex-col items-center gap-4">
              <p>Connect your wallet to mint</p>
              <WalletButton className="btn btn-primary" />
            </div>
          </div>
        </div>
      </div>
    );
  } else {
    return (
      <div>
        <AppHero 
          title="Storymint" 
          subtitle="Lock 1 SOL when you mint an NFT, and get it back when you dissolve it."
        >
          <div className="flex flex-col items-center gap-4">
            <button 
              onClick={handleMint}
              className="btn btn-primary"
              disabled={isMinting}
            >
              {isMinting ? 'Minting...' : 'Mint NFT'}
            </button>
          </div>
        </AppHero>
          {isLoading ? (
            <div className="flex justify-center items-center p-8">
              <div className="inline-flex items-center gap-2">
                <span className="loading loading-spinner loading-md"></span>
                <span>Loading assets...</span>
              </div>
            </div>
          ) : (
           <AssetGrid 
            assets={assets}
            onBurn={handleBurn}
            burningAsset={burningAsset}
          />
        )}
      </div>
    );
  }
}
