import { AnchorIdl, rootNodeFromAnchorWithoutDefaultVisitor } from '@kinobi-so/nodes-from-anchor'
import { renderJavaScriptUmiVisitor, renderRustVisitor } from '@kinobi-so/renderers'
import { visit } from '@kinobi-so/visitors-core'
import anchorIdl from './anchor/target/idl/storymint.json'

async function generateClients(feature: string) {
  const node = rootNodeFromAnchorWithoutDefaultVisitor(anchorIdl as AnchorIdl)
  const prefix = `src/clients/${feature}/generated`

  const clients = [
    { type: 'Umi', dir: `${prefix}/umi/src`, renderVisitor: renderJavaScriptUmiVisitor },
    { type: 'Rust', dir: `${prefix}/rust/src`, renderVisitor: renderRustVisitor },
  ]

  for (const client of clients) {
    try {
      await visit(node, await client.renderVisitor(client.dir))
      console.log(`✅ Successfully generated ${client.type} client for directory: ${client.dir}!`)
    } catch (e) {
      console.error(`Error in ${client.renderVisitor.name}:`, e)
      throw e
    }
  }
}
const feature = process.env.FEATURE || 'localnet'
generateClients(feature)
