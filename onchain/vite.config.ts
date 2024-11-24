import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { nodePolyfills } from 'vite-plugin-node-polyfills'
import viteTsconfigPaths from 'vite-tsconfig-paths' // https://vitejs.dev/config/

const API_URL = process.env.NODE_ENV === 'production' ? 'TODO' : 'http://localhost:8000/api'

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks: {
          coral: ['@coral-xyz/anchor'],
          jotai: ['jotai'],
          react: ['react', 'react-dom'],
          reactHotToast: ['react-hot-toast'],
          reactRouter: ['react-router', 'react-router-dom'],
          solanaWeb3: ['@solana/web3.js'],
          solanaWalletAdapters: [
            '@solana/wallet-adapter-base',
            '@solana/wallet-adapter-react',
            '@solana/wallet-adapter-react-ui',
          ],
          tabler: ['@tabler/icons-react'],
          tanstack: ['@tanstack/react-query'],
        },
      },
    },
  },
  define: {
    global: 'globalThis',
  },
  plugins: [
    viteTsconfigPaths(),
    react(),
    nodePolyfills({
      protocolImports: true,
      globals: {
        process: true,
      },
    }),
  ],
  server: {
    proxy: {
      '^/api/.*': {
        target: API_URL,
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, ''),
      },
    },
  },
})
