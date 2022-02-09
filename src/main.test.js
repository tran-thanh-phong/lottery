beforeAll(async function () {
  // NOTE: nearlib and nearConfig are made available by near-cli/test_environment
  const near = await nearlib.connect(nearConfig)
  window.accountId = nearConfig.contractName
  window.contract = await near.loadContract(nearConfig.contractName, {
    viewMethods: ['get_owner_id', 'get_account_balance'],
    changeMethods: ['new', 'deposit', 'create_jackpot', 'get_jackpots'],
    sender: window.accountId
  })

  window.walletConnection = {
    requestSignIn() {
    },
    signOut() {
    },
    isSignedIn() {
      return true
    },
    getAccountId() {
      return window.accountId
    }
  }

  console.log('Initializing contract...');
  await initContract();
  console.log('Contract initialized.');
})

async function initContract() {
  try {
    await window.contract.new({ owner_id: window.accountId });
  } catch (e) {
    if (!/Already initialized!/.test(e.toString())) {
      throw e;
    }
  }
}

const nearAPI = require('near-api-js');
const { KeyPair, Account, utils: { format: { parseNearAmount }} } = nearAPI;

const GAS = "200000000000000";

test('get_owner_id', async () => {
  const ownerId = await window.contract.get_owner_id()
  expect(ownerId).toEqual(window.accountId)
})

test('get_account_balance', async () => {
  const balance = await window.contract.get_account_balance({ account_id: window.accountId })
  expect('0').toEqual(balance)
})

test('deposit', async () => {
  const depositAmount = parseNearAmount('1');
  await window.contract.deposit({}, GAS, depositAmount)
  const balance = await window.contract.get_account_balance({ account_id: window.accountId })
  expect(depositAmount).toEqual(balance)
})