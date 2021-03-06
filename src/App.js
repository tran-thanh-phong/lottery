import 'regenerator-runtime/runtime'
import React from 'react'
import { login, logout, formatTime, formatNumber } from './utils'
import './global.css'

import getConfig from './config'
const { networkId } = getConfig(process.env.NODE_ENV || 'development')
const nearAPI = require('near-api-js');
const { KeyPair, Account, utils: { format: { parseNearAmount, formatNearAmount }} } = nearAPI;

const GAS = "200000000000000";

export default function App() {
  // use React Hooks to store greeting in component state
  const [claimableAmount, setClaimableAmount] = React.useState(0)
  const [jackpots, setJackpots] = React.useState([])
  const [accountTickets, setAccountTickets] = React.useState([])
  const [accountInfo, setAccountInfo] = React.useState({})

  // when the user has not yet interacted with the form, disable the button
  const [buttonDisabled, setButtonDisabled] = React.useState(true)

  // after submitting the form, we want to show Notification
  const [showNotification, setShowNotification] = React.useState(false)

  // The useEffect hook can be used to fire side-effects during render
  // Learn more: https://reactjs.org/docs/hooks-intro.html
  React.useEffect(
    () => {
      // in this case, we only care to query the contract when signed in
      if (window.walletConnection.isSignedIn()) {
        
      }

      getClaimableAmountFromContract()
      getJackpotListFromContract()
      getAccountInfoFromContract()
    },

    // The second argument to useEffect tells React when to re-run the effect
    // Use an empty array to specify "only run on first render"
    // This works because signing into NEAR Wallet reloads the page
    []
  )

  function getClaimableAmountFromContract() {
    if (window.walletConnection.isSignedIn()) {
      // window.contract is set by initContract in index.js
      window.contract.get_account_balance({ account_id: window.accountId })
        .then(result => {
          setClaimableAmount(formatNearAmount(result))
        })
    }
  }

  function getJackpotListFromContract() {
    window.contract.get_jackpots({})
      .then(result => {
        console.log('Get jackpot list from contract: ', result)
        setJackpots(result)

        getAccountTicketsFromContract(result)
      })
  }

  function getAccountInfoFromContract() {
    window.contract.get_account_info_or_default({account_id: window.accountId})
      .then(result => {
        console.log('Get Account Info from contract: ', result)
        setAccountInfo(result)
      })
  }

  function getAccountTicketsFromContract(jackpots) {
    window.contract.get_account_tickets({account_id: window.accountId})
      .then(result => {
        console.log('Get Account Tickets from contract: ', result)
        result.map(item => {
          let jackpot = jackpots.find(j => j.startTime <= item.createdTime && (!j.endTime || item.createdTime <= j.endTime))
          console.log('Jackpots', jackpots , 'Jackpot', jackpot)
          item.jackpotId = jackpot.id
          item.result = jackpot.winTicketIds.includes(item.id) ? 'Won' : jackpot.status == 'Open' ? 'Waiting' : 'Loss'
        })
        console.log('Get Account Tickets from contract: ', result)
        setAccountTickets(result)
      })
  }

  function hasOpenJackpot() {
    if (!jackpots || jackpots.length == 0) {
      return false
    }

    return jackpots[jackpots.length - 1].status == 'Open'
  }

  // if not signed in, return early with sign-in prompt
  if (!window.walletConnection.isSignedIn()) {
    return (
      <main>
        <h1>Welcome to The Lottery!</h1>
        <p style={{ textAlign: 'center'}}>
          Please sign in:
        </p>
        <p style={{ textAlign: 'center', marginTop: '1em' }}>
          <button onClick={login}>Sign in</button>
        </p>
      </main>
    )
  }

  return (
    // use React Fragment, <>, to avoid wrapping elements in unnecessary divs
    <>
      <button className="link" style={{ float: 'right' }} onClick={logout}>
        Sign out
      </button>
      <main>
        <h1>
          <label
            htmlFor="depositInput"
            style={{
              color: 'var(--secondary)',
              borderBottom: '2px solid var(--secondary)'
            }}
          >
          </label>
          {' '/* React trims whitespace around tags; insert literal space character when needed */}
          {window.accountId}
        </h1>
        <p>
          Wallet balance: <strong>{window.accountBalance} NEAR</strong> 
        </p>
        <form onSubmit={async event => {
          event.preventDefault()

          // get elements from the form using their id attribute
          const { fieldset, depositInput } = event.target.elements

          // hold onto new user-entered value from React's SynthenticEvent for use after `await` call
          const depositAmount = depositInput.value

          // disable the form while the value gets updated on-chain
          fieldset.disabled = true

          try {
            // make an update call to the smart contract
            await window.contract.deposit({}, GAS, parseNearAmount(depositAmount))
          } catch (e) {
            alert(
              'Something went wrong! ' +
              'Maybe you need to sign out and back in? ' +
              'Check your browser console for more info.'
            )
            throw e
          } finally {
            // re-enable the form, whether the call succeeded or failed
            fieldset.disabled = false
          }

          getClaimableAmountFromContract()

          // show Notification
          setShowNotification(true)

          // remove Notification again after css animation completes
          // this allows it to be shown again next time the form is submitted
          setTimeout(() => {
            setShowNotification(false)
          }, 11000)
        }}>
          <fieldset id="fieldset">
            <label
              htmlFor="depositInput"
              style={{
                display: 'block',
                color: 'var(--gray)',
                marginBottom: '0.5em'
              }}
            >
              Deposit amount
            </label>
            <div style={{ display: 'flex' }}>
              <input
                autoComplete="off"
                type="number"
                id="depositInput"
                onChange={e => setButtonDisabled(e.target.value <= 0)}
                style={{ flex: 1 }}
              />
              <button
                disabled={buttonDisabled}
                style={{ borderRadius: '0 5px 5px 0' }}
              >
                Deposit
              </button>
            </div>
          </fieldset>
        </form>
        
        <hr />
        {/* Normal Account Section */}
        <form onSubmit={async event => {
          event.preventDefault()

          try {
            // make an update call to the smart contract
            await window.contract.withdraw({}, GAS)
          } catch (e) {
            alert('Something went wrong!')
            throw e
          } finally {
            // re-enable the form, whether the call succeeded or failed
            //fieldset.disabled = false
          }

          getClaimableAmountFromContract()

        }}>
          <p>
            Claimable amount: <strong> {claimableAmount} NEAR</strong>
          </p>
          <button
            style={{ borderRadius: '0 5px 5px 0' }}
            disabled={claimableAmount <= 0}
          >
            Claim
          </button>
        </form>

        <form onSubmit={async event => {
          event.preventDefault()

          const { num1, num2, num3, num4, num5, num6 } = event.target.elements
          const numbers = [parseInt(num1.value), parseInt(num2.value), parseInt(num3.value), parseInt(num4.value), parseInt(num5.value), parseInt(num6.value)];

          try {
            // make an update call to the smart contract
            await window.contract.buy_ticket({ picked_numbers: numbers }, GAS)
          } catch (e) {
            alert('Something went wrong!')
            throw e
          } finally {
            // re-enable the form, whether the call succeeded or failed
            //fieldset.disabled = false
          }
        }}>
          <p>
            Please choose 6 lucky numbers between 1 to 55. Ticket price equals to 1 NEAR.
          </p>
          <div style={{ display: 'flex', gap: '20px'}}>
            <input id='num1' style={{ width: '70px', padding: '10px', textAlign: 'center' }} type="text" min={1} max={55} />
            <input id='num2' style={{ width: '70px', padding: '10px', textAlign: 'center' }} type="text" min={1} max={55} />
            <input id='num3' style={{ width: '70px', padding: '10px', textAlign: 'center' }} type="text" min={1} max={55} />
            <input id='num4' style={{ width: '70px', padding: '10px', textAlign: 'center' }} type="text" min={1} max={55} />
            <input id='num5' style={{ width: '70px', padding: '10px', textAlign: 'center' }} type="text" min={1} max={55} />
            <input id='num6' style={{ width: '70px', padding: '10px', textAlign: 'center' }} type="text" min={1} max={55} />
          </div>
          <button
            style={{ borderRadius: '0 5px 5px 0', marginTop: '10px', marginBottom: '10px' }}
            disabled={claimableAmount <= 0 || !hasOpenJackpot()}
          >
            Buy Ticket
          </button>
        </form>

        <div>
          <br/>
          <h3>Ticket List</h3>
          <table>
            <thead>
              <tr>
                <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Id</td>
                <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Jackpot</td>
                <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Time</td>
                <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Numbers</td>
                <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Result</td>
              </tr>
            </thead>
            <tbody>
              {accountTickets.map(item =>
                <tr key={'at_' + item.id.toString()}>
                  <td style={{paddingRight: '10px', paddingLeft: '10px'}}>{item.id}</td>
                  <td style={{paddingRight: '10px', paddingLeft: '10px'}}>{item.jackpotId}</td>
                  <td style={{paddingRight: '10px', paddingLeft: '10px'}}>{formatTime(item.createdTime)}</td>
                  <td style={{paddingRight: '10px', paddingLeft: '10px'}}>
                    {item.pickedNumbers.map((number, index) =>
                      <span key={index} style={{marginRight: '5px'}}>{formatNumber(number)}</span>
                    )}
                  </td>
                  <td style={{paddingRight: '10px', paddingLeft: '10px'}}>{item.result}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
        <hr />

        {/* System Section */}
        <h2>Contract Information</h2>
        <div>
          <h3>Jackpot List</h3>
          {jackpots.map(item => 
            <div key={'j_' + item.id}>
              <div style={{display: 'flex', flexDirection: 'column'}}>
                <table>
                  <tbody>
                    <tr>
                      <td>Jackpot Id:</td>
                      <td>{item.id}</td>
                    </tr>
                    <tr>
                      <td>Ticket Price:</td>
                      <td><strong style={{whiteSpace: 'nowrap'}}>{formatNearAmount(item.ticketPrice)} NEAR</strong></td>
                    </tr>
                    <tr>
                      <td>Prize:</td>
                      <td><strong style={{whiteSpace: 'nowrap'}}>{formatNearAmount(item.lockedAmount)} NEAR</strong></td>
                    </tr>
                    <tr>
                      <td>Tickets:</td>
                      <td><strong>{item.noOfTickets}</strong></td>
                    </tr>
                    <tr>
                      <td>Status:</td>
                      <td>{item.status}</td>
                    </tr>
                    <tr>
                      <td>Open Time:</td>
                      <td>{formatTime(item.startTime)}</td>
                    </tr>
                    {item.endTime && 
                      <tr>
                        <td>Close Time:</td>
                        <td>{formatTime(item.endTime)}</td>
                      </tr>
                    }
                  </tbody>
                </table>
              </div>
              
              <div>
                <label>Drawing Result: </label>
                <table>
                  <thead>
                    <tr>
                      <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Id</td>
                      <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Time</td>
                      <td style={{paddingRight: '10px', paddingLeft: '10px'}}>Numbers</td>
                    </tr>
                  </thead>
                  <tbody>
                    {item.drawedResults.map((result, index) => 
                      <tr key={index}>
                        <td style={{paddingRight: '10px', paddingLeft: '10px'}}>{index + 1}</td>
                        <td style={{paddingRight: '10px', paddingLeft: '10px'}}>{formatTime(result.createdTime)}</td>
                        <td style={{paddingRight: '10px', paddingLeft: '10px'}}> 
                          {result.drawedNumbers.map((number, index) =>
                            <span key={index} style={{marginRight: '5px'}}>{formatNumber(number)}</span>
                          )}
                        </td>
                      </tr>  
                    )}
                  </tbody>
                </table>
                
              </div>
              <br/>
            </div>
          )}
          
        </div>
        <br/>
        <hr />
        {/* Contract Owner Section */}
        { window.contractOwnerId == window.accountId && 
          <>
            <h2>Contract Owner</h2>
            <p>Let do some oprations</p>
            <div>
              <button
                style={{ borderRadius: '5px', marginRight: '10px' }}
                disabled={hasOpenJackpot()}
                onClick={async () => {
                  try {
                    // make an update call to the smart contract
                    console.log('Jackpot is creating...')
                    await window.contract.create_jackpot({}, GAS, parseNearAmount('10'))
                    console.log('Jackpot created.')
                  } catch (e) {
                    alert('Something went wrong!')
                    throw e
                  } finally {

                  }
                }}
              >
                Create Jackpot
              </button>
            
              <button
                style={{ borderRadius: '5px', marginRight: '10px' }}
                disabled={!hasOpenJackpot()}
                onClick={async () => {
                  try {
                    // make an update call to the smart contract
                    console.log('Jackpot is drawing...')
                    let result = await window.contract.draw_jackpot({force_win: false}, GAS)
                    console.log('Jackpot drawn with result: ', result)
                  } catch (e) {
                    alert('Something went wrong!')
                    throw e
                  } finally {

                  }
                }}
              >
                Draw Jackpot
              </button>
              <button
                style={{ borderRadius: '5px', marginRight: '10px' }}
                disabled={!hasOpenJackpot()}
                onClick={async () => {
                  try {
                    // make an update call to the smart contract
                    console.log('Jackpot is drawing...')
                    let result = await window.contract.draw_jackpot({force_win: true}, GAS)
                    console.log('Jackpot drawn with result: ', result)
                  } catch (e) {
                    alert('Something went wrong!')
                    throw e
                  } finally {

                  }
                }}
              >
                Draw to Win
              </button>
            </div>
              
          </>
        }

      </main>
      {showNotification && <Notification />}
    </>
  )
}

// this component gets rendered by App after the form is submitted
function Notification() {
  const urlPrefix = `https://explorer.${networkId}.near.org/accounts`
  return (
    <aside>
      <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.accountId}`}>
        {window.accountId}
      </a>
      {' '/* React trims whitespace around tags; insert literal space character when needed */}
      called method: 'set_greeting' in contract:
      {' '}
      <a target="_blank" rel="noreferrer" href={`${urlPrefix}/${window.contract.contractId}`}>
        {window.contract.contractId}
      </a>
      <footer>
        <div>??? Succeeded</div>
        <div>Just now</div>
      </footer>
    </aside>
  )
}

