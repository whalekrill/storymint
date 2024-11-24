import { UiLayout } from '@/components/ui/ui-layout'
import { lazy } from 'react'
import { Navigate, RouteObject, useRoutes } from 'react-router-dom'
const Storymint = lazy(() => import('../components/storymint/storymint'))

const links: { label: string; path: string }[] = [
  { label: 'Storymint', path: '/storymint' },
]

const routes: RouteObject[] = [
  { path: '/', element: <Storymint /> },
]

export function AppRoutes() {
  const router = useRoutes([
    ...routes,
    { path: '*', element: <Navigate to={'/'} replace={true} /> },
  ])
  return <UiLayout links={links}>{router}</UiLayout>
}
