import { UiLayout } from '@/components/ui/ui-layout'
import { lazy } from 'react'
import { useEffect } from 'react'
import { useWallet } from '@solana/wallet-adapter-react'
import { Navigate, RouteObject, useRoutes } from 'react-router-dom'
import { signInWithSolana } from '@/components/solana/signinWithSolana'
const AccountListFeature = lazy(() => import('../components/account/account-list-feature'))
const AccountDetailFeature = lazy(() => import('../components/account/account-detail-feature'))
const ClusterFeature = lazy(() => import('../components/cluster/cluster-feature'))
const BasicFeature = lazy(() => import('../components/basic/basic-feature'))
const DashboardFeature = lazy(() => import('../components/dashboard/dashboard-feature'))

const links: { label: string; path: string }[] = [
  { label: 'Account', path: '/account' },
  { label: 'Clusters', path: '/clusters' },
  { label: 'Basic Program', path: '/basic' },
]

const routes: RouteObject[] = [
  { path: '/account/', element: <AccountListFeature /> },
  { path: '/account/:address', element: <AccountDetailFeature /> },
  { path: '/basic', element: <BasicFeature /> },
  { path: '/clusters', element: <ClusterFeature /> },
]

export function AppRoutes() {

  // region custom
  // const wallet = useWallet()
  // useEffect(() => {
  //   async function autoSignIn() {
  //     if (wallet.connected) {
  //       await signInWithSolana(wallet)
  //     }
  //   }
  //   autoSignIn()
  // }, [wallet])
  // endregion

  const router = useRoutes([
    { index: true, element: <Navigate to={'/dashboard'} replace={true} /> },
    { path: '/dashboard', element: <DashboardFeature /> },
    ...routes,
    { path: '*', element: <Navigate to={'/dashboard'} replace={true} /> },
  ])
  return <UiLayout links={links}>{router}</UiLayout>
}
