export function ExplorerLink({ 
  path, 
  label, 
  className,
  cluster = 'devnet' 
}: { 
  path: string; 
  label: string; 
  className?: string;
  cluster?: 'devnet' | 'mainnet-beta' | 'testnet';
}) {
  const explorerUrl = `https://explorer.solana.com/${path}?cluster=${cluster}`
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
