// import * as localnetModules from '../../clients/localnet/generated/umi/src'
import * as devnetModules from '../../clients/devnet/generated/umi/src'

export default async function umiModules() {
  // if (import.meta.env.MODE === 'localnet') {
  //   return {
  //     createStorymintProgram: localnetModules.createStorymintProgram,
  //     mintAsset: localnetModules.mintAsset,
  //     burnAndWithdraw: localnetModules.burnAndWithdraw,
  //   }
  // }

  return {
    createStorymintProgram: devnetModules.createStorymintProgram,
    mintAsset: devnetModules.mintAsset,
    burnAndWithdraw: devnetModules.burnAndWithdraw,
  }
}
