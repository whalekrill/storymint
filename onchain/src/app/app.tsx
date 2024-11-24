import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { SolanaProvider } from '../components/solana/solana-provider'
import { AppRoutes } from './app-routes'

const client = new QueryClient()

export function App() {
  return (
    <QueryClientProvider client={client}>
        <SolanaProvider>
          <AppRoutes />
        </SolanaProvider>
    </QueryClientProvider>
  )
}
