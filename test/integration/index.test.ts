import { SetHookFlags } from '@transia/xrpl'

import {
  type SetHookParams,
  type XrplIntegrationTestContext,
  Xrpld,
  clearAllHooksV3,
  clearHookStateV3,
  createHookPayload,
  hexNamespace,
  type iHook,
  serverUrl as localServerUrl,
  setHooksV3,
  setupClient,
  teardownClient,
} from '@transia/hooks-toolkit'

const namespace = 'namespace'

const serverUrl = localServerUrl

describe('test', () => {
  let testContext: XrplIntegrationTestContext

  beforeAll(async () => {
    testContext = await setupClient(serverUrl)
    const hook = createHookPayload({
      version: 0,
      createFile: 'index',
      namespace: namespace,
      flags: SetHookFlags.hsfOverride,
      hookOnArray: ['Invoke'],
    })
    await setHooksV3({
      client: testContext.client,
      seed: testContext.alice.seed,
      hooks: [{ Hook: hook }],
    } as SetHookParams)
  })

  afterAll(async () => {
    const clearHook: iHook = {
      Flags: SetHookFlags.hsfNSDelete,
      HookNamespace: hexNamespace(namespace),
    }
    await clearHookStateV3({
      client: testContext.client,
      seed: testContext.alice.seed,
      hooks: [{ Hook: clearHook }],
    } as SetHookParams)
    await clearAllHooksV3({
      client: testContext.client,
      seed: testContext.alice.seed,
    } as SetHookParams)
    await teardownClient(testContext)
  })

  it('', async () => {
    const response = await Xrpld.submit(testContext.client, {
      tx: {
        TransactionType: 'Invoke',
        Account: testContext.alice.address,
      },
      wallet: testContext.alice,
    })
    console.log(response.meta)
    expect(response.meta).toHaveProperty('HookExecutions')
  })
})
