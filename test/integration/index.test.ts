// src
import {
  ExecutionUtility,
  type SetHookParams,
  Xrpld,
  clearAllHooksV3,
  clearHookStateV3,
  createHookPayload,
  hexNamespace,
  type iHook,
  setHooksV3,
} from '@transia/hooks-toolkit'
// xrpl-helpers
import {
  type XrplIntegrationTestContext,
  close,
  serverUrl,
  setupClient,
  teardownClient,
} from '@transia/hooks-toolkit/dist/npm/src/libs/xrpl-helpers'
import {
  type AccountSet,
  AccountSetAsfFlags,
  SetHookFlags,
  type TransactionMetadata,
  type URITokenCreateSellOffer,
  type URITokenMint,
  convertStringToHex,
  xrpToDrops,
} from '@transia/xrpl'
import { hashURIToken } from '@transia/xrpl/dist/npm/utils/hashes'

// AutoTransfer: ACCEPT: success

describe('autotransfer', () => {
  let testContext: XrplIntegrationTestContext

  beforeAll(async () => {
    testContext = await setupClient(serverUrl)
    const hookWallet = testContext.hook1
    const hook = createHookPayload({
      version: 0,
      createFile: 'index',
      namespace: 'autotransfer',
      flags: SetHookFlags.hsfCollect + SetHookFlags.hsfOverride,
      hookOnArray: ['URITokenCreateSellOffer'],
    })
    await setHooksV3({
      client: testContext.client,
      seed: hookWallet.seed,
      hooks: [{ Hook: hook }],
    } as SetHookParams)
  })
  afterAll(async () => {
    await clearAllHooksV3({
      client: testContext.client,
      seed: testContext.hook1.seed,
    } as SetHookParams)

    const clearHook = {
      Flags: SetHookFlags.hsfNSDelete,
      HookNamespace: hexNamespace('autotransfer'),
    } as iHook
    await clearHookStateV3({
      client: testContext.client,
      seed: testContext.hook1.seed,
      hooks: [{ Hook: clearHook }],
    } as SetHookParams)
    teardownClient(testContext)
  })

  it('autotransfer - success', async () => {
    const hookWallet = testContext.hook1
    const aliceWallet = testContext.alice

    // AccountSet: Hook Acct
    const asTx: AccountSet = {
      TransactionType: 'AccountSet',
      Account: hookWallet.classicAddress,
      SetFlag: AccountSetAsfFlags.asfTshCollect,
    }
    await Xrpld.submit(testContext.client, {
      wallet: hookWallet,
      tx: asTx,
    })
    // biome-ignore lint/style/useTemplate: <explanation>
    const uritokenString = 'ipfs://autotransfer' + Math.random()
    // URITokenMint
    const mintTx: URITokenMint = {
      TransactionType: 'URITokenMint',
      Account: aliceWallet.classicAddress,
      URI: convertStringToHex(uritokenString),
    }
    await Xrpld.submit(testContext.client, {
      wallet: aliceWallet,
      tx: mintTx,
    })
    const uriTokenID = hashURIToken(aliceWallet.classicAddress, uritokenString)

    // URITokenCreateSellOffer
    const builtTx1: URITokenCreateSellOffer = {
      TransactionType: 'URITokenCreateSellOffer',
      Account: aliceWallet.classicAddress,
      Amount: xrpToDrops(0),
      Destination: hookWallet.classicAddress,
      URITokenID: uriTokenID,
    }
    const result1 = await Xrpld.submit(testContext.client, {
      wallet: aliceWallet,
      tx: builtTx1,
    })
    await close(testContext.client)

    const hookExecutions1 = await ExecutionUtility.getHookExecutionsFromMeta(
      testContext.client,
      result1.meta as TransactionMetadata,
    )
    expect(hookExecutions1.executions[0].HookReturnString).toMatch('autotransfer: Tx emitted success')
  })
})
