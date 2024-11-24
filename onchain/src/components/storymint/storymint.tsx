import { useState, useMemo } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { createUmi } from '@metaplex-foundation/umi-bundle-defaults'
import { WalletButton } from '../solana/solana-provider';
import { AppHero } from '../ui/ui-layout';
import { createStorymintProgram } from '../../../clients/generated/umi/src/';
import { mplCore } from '@metaplex-foundation/mpl-core';
import { publicKey } from '@metaplex-foundation/umi'
import { walletAdapterIdentity } from '@metaplex-foundation/umi-signer-wallet-adapters';
import { mint } from './instructions';
import AssetGrid from './assetGrid';


export default function Storymint() {
  const wallet = useWallet();

  const [shouldReloadAssets, setShouldReloadAssets] = useState(false);

  const umi = useMemo(() => {
    return createUmi("https://api.devnet.solana.com")
    .use(mplCore())
    .use({
      install(umi) {
        umi.programs.add(createStorymintProgram());
      },
    })
    .use(walletAdapterIdentity(wallet));
  }, [wallet]);


  const handleMint = async () => {
    if (wallet.connected && wallet.publicKey) {
      await mint(umi, publicKey(wallet.publicKey));
      setShouldReloadAssets(true);
    }
  }

  return (
    <div>
      {wallet.connected ? (
        <div>
        <AppHero title="Storymint" subtitle="Lock 1 SOL when you mint an NFT, and get it back when you dissolve it.">
          <div className="flex flex-col items-center gap-4">
            <button 
              onClick={handleMint}
              className="btn btn-primary"
            >
              Mint Asset
            </button>
          </div>
        </AppHero>
        <AssetGrid umi={umi} shouldReloadAssets={shouldReloadAssets} setShouldReloadAssets={setShouldReloadAssets} />
        </div>
      ) : (
        <div className="max-w-4xl mx-auto">
          <div className="hero py-[64px]">
            <div className="hero-content text-center">
              <div className="flex flex-col items-center gap-4">
                <p>Connect your wallet to mint</p>
                <WalletButton className="btn btn-primary" />
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
