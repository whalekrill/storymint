import { ReactNode, useCallback} from 'react'
import { WalletError } from '@solana/wallet-adapter-base'
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react'
import { WalletModalProvider, WalletMultiButton } from '@solana/wallet-adapter-react-ui'
// import type { Adapter } from '@solana/wallet-adapter-base'
// import { signInWithSolana } from './signInWithSolana'

import('@solana/wallet-adapter-react-ui/styles.css')

export const WalletButton = WalletMultiButton


export function SolanaProvider({ children }: { children: ReactNode }) {
  const endpoint = 'https://api.devnet.solana.com'
  const onError = useCallback((error: WalletError) => {
    console.error(error)
  }, [])
  
  // region custom
  // const useAutoSignIn = useCallback(async (adapter: Adapter) => {
  //   return await signInWithSolana(adapter)
  // }, [])
  // endregion
  // <WalletProvider wallets={[]} onError={onError} autoConnect={useAutoSignIn}>

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={[]} onError={onError} autoConnect={true}>
        <WalletModalProvider>{children}</WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  )
}
