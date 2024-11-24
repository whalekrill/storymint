export function ExplorerLink({ path, label, className }: { path: string; label: string; className?: string }) {
  const explorerUrl =  `https://explorer.solana.com/${path}`
  return (
    <a
      href={explorerUrl}
      target="_blank"
      rel="noopener noreferrer"
      className={className ? className : `link font-mono`}
    >
      {label}
    </a>
  )
}
