import { connect, Contract, keyStores, WalletConnection } from 'near-api-js'
import getConfig from './config'

const nearConfig = getConfig(process.env.NODE_ENV || 'development')
const { formatNearAmount } = require('near-api-js/lib/utils/format');

// Initialize contract & set global variables
export async function initContract() {
  // Initialize connection to the NEAR testnet
  const near = await connect(Object.assign({ deps: { keyStore: new keyStores.BrowserLocalStorageKeyStore() } }, nearConfig))

  // Initializing Wallet based Account. It can work with NEAR testnet wallet that
  // is hosted at https://wallet.testnet.near.org
  window.walletConnection = new WalletConnection(near)

  // Getting the Account ID. If still unauthorized, it's just empty string
  window.accountId = window.walletConnection.getAccountId()

  // Initializing our contract APIs by contract name and configuration
  window.contract = await new Contract(window.walletConnection.account(), nearConfig.contractName, {
    // View methods are read only. They don't modify the state, but usually return some value.
    viewMethods: ['get_owner_id', 'get_account_balance', 'get_jackpots', 'get_account_info_or_default', 'get_account_tickets'],
    // Change methods can modify the state. But you don't receive the returned value when called.
    changeMethods: ['new', 'set_owner_id', 'create_jackpot', 'deposit', 'withdraw', 'buy_ticket', 'draw_jackpot'],
  })

  //await initializeContract();

  // Account balance
  console.log("Account Id: ", window.accountId);
  if (window.accountId) {
    const account = await near.account(window.accountId);
    window.accountBalance = formatNearAmount((await account.getAccountBalance()).available, 2);  
  }

  window.contractOwnerId = await window.contract.get_owner_id({})
}

export async function initializeContract() {
  try {
    await window.contract.new({ owner_id: window.accountId });
  } catch (e) {
    if (!/Already initialized!/.test(e.toString())) {
      //throw e;
      console.log(e);
    }
  }
}

export function logout() {
  window.walletConnection.signOut()
  // reload page
  window.location.replace(window.location.origin + window.location.pathname)
}

export function login() {
  // Allow the current app to make calls to the specified contract on the
  // user's behalf.
  // This works by creating a new access key for the user's account and storing
  // the private key in localStorage.
  window.walletConnection.requestSignIn(nearConfig.contractName)
}

export function formatTime(unix1) {
  let unix = unix1 / 1000000
  let date = new Date(unix)

  var year = date.getYear().toString()
  var month = formatNumber(date.getMonth())
  var day = formatNumber(date.getDay())
  var hours = formatNumber(date.getHours())
  var minutes = formatNumber(date.getMinutes())
  var seconds = formatNumber(date.getSeconds())

  var formattedTime = year + '-' + month + '-' + day + ' ' + hours + ':' + minutes + ':' + seconds

  // console.log('Time: ', formattedTime, 'Unix Time: ', unix, 'From: ', unix1)
  // console.log('Date: ', date)

  return date.toISOString()
}

export function formatNumber(number) {
  return number > 9 ? number.toString() : '0' + number
}