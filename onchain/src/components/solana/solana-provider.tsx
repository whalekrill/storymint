import { WalletError } from '@solana/wallet-adapter-base'
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react'
import { WalletModalProvider, WalletMultiButton } from '@solana/wallet-adapter-react-ui'
import type { Adapter } from '@solana/wallet-adapter-base'
import { ReactNode, useCallback} from 'react'
// import { signInWithSolana } from './signInWithSolana'

import('@solana/wallet-adapter-react-ui/styles.css')

export const WalletButton = WalletMultiButton


export function SolanaProvider({ children }: { children: ReactNode }) {
  const endpoint = 'https://api.devnet.solana.com'
  const onError = useCallback((error: WalletError) => {
    console.error(error)
  }, [])
  
  // region custom
  const useAutoSignIn = useCallback(async (adapter: Adapter) => {
    // return await signInWithSolana(adapter)
    return false
  }, [])
  // endregion

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={[]} onError={onError} autoConnect={useAutoSignIn}>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  )
}
