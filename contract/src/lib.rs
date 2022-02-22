/*
 * This is an example of a Rust smart contract with two simple, symmetric functions:
 *
 * 1. set_greeting: accepts a greeting, such as "howdy", and records it for the user (account_id)
 *    who sent the request
 * 2. get_greeting: accepts an account_id and returns the greeting saved for it, defaulting to
 *    "Hello"
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, Balance, Promise, Timestamp};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::json_types::{U128};
use serde::ser::{Serialize, Serializer, SerializeStruct};
use std::option::Option;
use std::convert::TryFrom;

setup_alloc!();

const ONE_NEAR: u128 = 1_000_000_000_000_000_000_000_000;
const MAX_DRAWING_NUMBER: u8 = 55;

type TicketId = u64;
type JackpotId = u32;

use crate::utils::*;
pub use crate::account::*;
pub use crate::ticket::*;
pub use crate::jackpot::*;

mod utils;
mod account;
mod ticket;
mod jackpot;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DrawingResult {
    drawed_numbers: [u8; 6],
    created_time: Timestamp
}

impl Serialize for DrawingResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("DrawingResult", 1)?;
        state.serialize_field("drawedNumbers", &self.drawed_numbers)?;
        state.serialize_field("createdTime", &self.created_time)?;
        state.end()
    }
}

impl Default for DrawingResult {
    fn default() -> Self {
        let mut drawed_numbers: [u8; 6] = [0, 0, 0, 0, 0, 0];
        let mut ran_no: u8 = 1;

        for i in 0..6 {
            loop {
                ran_no += 1;
                let number = u8::try_from(get_random_number(MAX_DRAWING_NUMBER.into(), ran_no.into())).ok().unwrap();
                let mut j = 0;
                while j < i {
                    if drawed_numbers[j] == number {
                        break;
                    }
                    j += 1;
                }

                if j >= i {
                    drawed_numbers[i] = number;
                    break;
                }
            }
        }

        drawed_numbers.sort();

        Self {
            drawed_numbers,
            created_time: get_time_now(),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Lottery {
    owner_id: AccountId,
    // Store the NEAR balance or user. Using UnorderedMap in order to easily retrival all items inside the map
    account_infoes: UnorderedMap<AccountId, AccountInfo>,
    jackpots: Vector<Jackpot>,
    tickets: UnorderedMap<TicketId, Ticket>,
}

impl Default for Lottery {
    fn default() -> Self {
        panic!("Should be initialized before usage.");
    }
}

#[near_bindgen]
impl Lottery {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(&owner_id.as_bytes()), "Invalid owner account!");
        assert!(!env::state_exists(), "Already initialized!");

        env::log(format!("Creating a Lottery with owner id '{}'", &owner_id).as_bytes());

        Self {
            owner_id,
            account_infoes: UnorderedMap::new(b"account_info".to_vec()),
            jackpots: Vector::new(b"jackpot".to_vec()),
            tickets: UnorderedMap::new(b"ticket".to_vec()),
        }
    }

    pub fn has_initialized(&self) -> bool {
        env::state_exists()
    }

    pub fn get_number(&self) -> u8{
        let number: u8 = u8::try_from(get_random_number(MAX_DRAWING_NUMBER.into(), 0)).ok().unwrap();
        println!("The random number is: {}", number);
        number
    }
    
    pub fn set_owner_id(&mut self, owner_id: AccountId) {
        let current_owner_id = env::signer_account_id();
        assert_eq!(current_owner_id, self.owner_id, "Only owner can change ownership.");

        self.owner_id = owner_id;
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }

    pub fn get_account_info_or_default(&self, account_id: &AccountId) -> AccountInfo {
        match self.account_infoes.get(&account_id) {
            None => AccountInfo::new((&account_id).to_string()),
            Some(a) => a,
        }
    }

    pub fn get_account_balance(&self, account_id: &AccountId) -> U128 {
        self.get_account_info_or_default(&account_id).balance.into()
    }

    pub fn get_account_tickets(&self, account_id: &AccountId) -> Vec<Ticket> {
        let ticket_ids = self.get_account_info_or_default(&account_id).ticket_ids.to_vec();
        let mut tickets = Vec::new();

        for i in 0..ticket_ids.len() {
            let ticket_id = ticket_ids.get(i).unwrap();
            let ticket = self.tickets.get(ticket_id).unwrap();
            tickets.push(ticket);
        }

        tickets
    }

    #[payable]
    pub fn deposit(&mut self) {
        let account_id = env::signer_account_id();
        let deposit_amount = env::attached_deposit();

        let mut account_info = self.get_account_info_or_default(&account_id);
        account_info.balance += deposit_amount;
        self.account_infoes.insert(&account_id, &account_info);
    }
    
    pub fn withdraw(&mut self) {
        let account_id = env::signer_account_id();

        // Get account balance
        let mut account_info = self.get_account_info_or_default(&account_id);
        let proceeds = account_info.balance;
        assert!(proceeds > 0, "nothing to withdraw");

        // Reset account balance
        account_info.balance = 0;
        self.account_infoes.insert(&account_id, &account_info);

        // Process withdrawal
        Promise::new(account_id).transfer(proceeds);
    }

    fn generate_ticket_id(&self) -> TicketId {
        (self.tickets.len() + 1).into()
    }

    fn generate_jackpot_id(&self) -> JackpotId {
        u32::try_from(self.jackpots.len() + 1).ok().unwrap()
    }

    pub fn get_latest_jackpot(&self) -> Option<Jackpot> {
        if self.jackpots.is_empty() {
            Option::None
        }
        else {
            let last_index = self.jackpots.len() - 1;
            self.jackpots.get(last_index)
        }
    } 

    pub fn get_jackpots(&self) -> Vec<Jackpot> {
        self.jackpots.to_vec()
    }

    fn update_latest_jackpot(&mut self, jackpot: &Jackpot) {
        self.jackpots.pop();
        self.jackpots.push(&jackpot);
    }

    #[payable]
    pub fn create_jackpot(&mut self, ticket_price: Option<u128>) {
        // Check account right (The signer must be the contract owner)
        let account_id = env::signer_account_id();
        assert!(account_id == self.owner_id, "The signer must be the contract owner.");

        // Check current jackpot status
        let latest_jackpot = self.get_latest_jackpot();
        match latest_jackpot {
            None => (),
            Some(j) => {
                assert!(matches!(j.get_status(), JackpotStatus::Close), "The latest jackpot is still open. Cannot create a new one!"); 
            }
        }

        let ticket_price = ticket_price.unwrap_or(ONE_NEAR);
        let initialized_amount = env::attached_deposit();
        
        // Create a new jackpot
        let id = self.generate_jackpot_id();
        let start_time = get_time_now();
        let jackpot = Jackpot::new(id, start_time, ticket_price, initialized_amount);

        self.jackpots.push(&jackpot);
    }

    pub fn buy_ticket(&mut self, picked_numbers: [u8; 6]) {
        let account_id = env::signer_account_id();
        let mut account_info = self.get_account_info_or_default(&account_id);

        // Check the current Jackpot is available for buying tickets
        let latest_jackpot = self.get_latest_jackpot();
        assert!(latest_jackpot.is_some() && matches!(latest_jackpot.as_ref().unwrap().get_status(), JackpotStatus::Open), "There is no open jackpot.");
        
        // Check user balance must be enough to by a ticket
        let mut latest_jackpot = latest_jackpot.unwrap();
        let ticket_price = latest_jackpot.ticket_price;
        assert!(account_info.balance >= ticket_price, "No balance to buy ticket!!!");

        // Create a ticket and add to list
        let ticket_id = self.generate_ticket_id();

        // Sort & validate numbers
        let mut picked_numbers = picked_numbers;
        picked_numbers.sort();

        for i in 0..6 {
            if picked_numbers[i] < 1 || picked_numbers[i] > 55 {
                panic!("The chosen number must be between 1 and 55.");
            }

            if i == 0 {
                continue;
            }

            assert!(picked_numbers[i-1] < picked_numbers[i], "The chosen numbers cannot be duplicated.");
        }

        let ticket = Ticket::new(&ticket_id, &account_id, &picked_numbers);
        self.tickets.insert(&ticket_id, &ticket);

        // Add the new ticket to current Jackpot
        latest_jackpot.ticket_ids.push(&ticket_id);

        // Descrease account balance and increase locked balance
        account_info.balance -= ticket_price;
        latest_jackpot.locked_amount += ticket_price;

        // Add ticket to current account
        account_info.ticket_ids.push(&ticket_id);

        self.account_infoes.insert(&account_id, &account_info);

        self.update_latest_jackpot(&latest_jackpot);
    }

    pub fn draw_jackpot(&mut self, force_win: bool) -> bool{
        // Check account right (The signer must be the contract owner)
        let account_id = env::signer_account_id();
        assert!(account_id == self.owner_id, "The signer must be the contract owner.");

        // Check current jackpot status
        let latest_jackpot = self.get_latest_jackpot();
        match latest_jackpot {
            None => (),
            Some(ref j) => {
                assert!(matches!(j.get_status(), JackpotStatus::Open), "The latest jackpot is closed. Cannot draw anymore!"); 
            }
        }

        let mut latest_jackpot = latest_jackpot.unwrap();

        let mut result = DrawingResult::default();

        if force_win {
            let no_of_tickets = latest_jackpot.ticket_ids.len();
            let lucky_ticket_index = get_random_number(no_of_tickets, 0) - 1;
            let lucky_ticket_id = latest_jackpot.ticket_ids.get(lucky_ticket_index).unwrap();
            let lucky_ticket = self.tickets.get(&lucky_ticket_id).unwrap();

            result.drawed_numbers = lucky_ticket.picked_numbers.clone();
        }

        // Add new result to list
        latest_jackpot.drawed_results.push(&result);

        // Check result
        let no_of_tickets = latest_jackpot.ticket_ids.len();
        for i in 0..no_of_tickets {
            let ticket_id = latest_jackpot.ticket_ids.get(i).unwrap();
            let ticket = self.tickets.get(&ticket_id).unwrap();

            if compare_numbers(&ticket.picked_numbers, &result.drawed_numbers) {
                // Add win ticket into list to track
                latest_jackpot.win_ticket_ids.push(&ticket_id);
            }
        }

        if !latest_jackpot.win_ticket_ids.is_empty() {
            // Finalize current jackpot
            latest_jackpot.end_time = Some(get_time_now());

            // Devide the price for winner
            let no_of_winners = latest_jackpot.win_ticket_ids.len();
            let price_amount = latest_jackpot.locked_amount / no_of_winners as Balance;
            for i in 0..no_of_winners {
                let ticket_id = latest_jackpot.win_ticket_ids.get(i).unwrap();
                let ticket = self.tickets.get(&ticket_id).unwrap();
                let account_id = ticket.account_id;

                let mut account_info = self.get_account_info_or_default(&account_id);

                // Increase account balance & descrease locked amount
                account_info.balance += price_amount;
                latest_jackpot.locked_amount -= price_amount;

                self.account_infoes.insert(&account_id, &account_info);
            }
        }

        self.update_latest_jackpot(&latest_jackpot);

        if matches!(latest_jackpot.get_status(), JackpotStatus::Close) {
            return true;
        }

        false
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    const DEPOSIT_AMOUNT: u128 = 10 * ONE_NEAR;

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 11,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: DEPOSIT_AMOUNT,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }

    #[test]
    fn get_default_owner_id() {
        let context = get_context(vec![], true);
        testing_env!(context);

        let contract = Lottery::new(String::from("alice_near"));
        
        assert_eq!(
            "alice_near".to_string(),
            contract.get_owner_id())
    }

    #[test]
    fn deposit() {
        let context = get_context(vec![], false);
        testing_env!(context);
        
        let mut contract = Lottery::new(String::from("carol_near"));

        contract.deposit();

        assert_eq!(
            U128::from(DEPOSIT_AMOUNT),
            contract.get_account_balance(&String::from("bob_near"))
        );

        assert_eq!(
            U128::from(0),
            contract.get_account_balance(&String::from("carol_near"))
        );
    }

    #[test]
    fn withdraw() {
        let context = get_context(vec![], false);
        testing_env!(context);
        
        let mut contract = Lottery::new(String::from("carol_near"));

        contract.deposit();

        assert_eq!(
            U128::from(DEPOSIT_AMOUNT),
            contract.get_account_balance(&String::from("bob_near"))
        );

        assert_eq!(
            U128::from(0),
            contract.get_account_balance(&String::from("carol_near"))
        );

        contract.withdraw();

        assert_eq!(
            U128::from(0),
            contract.get_account_balance(&String::from("bob_near"))
        );
    }

    #[test]
    #[should_panic(expected = "The latest jackpot is still open. Cannot create a new one!")]
    fn create_jackpot() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let mut contract = Lottery::new(String::from("bob_near"));

        assert!(contract.get_latest_jackpot().is_none());

        contract.create_jackpot(None, None);

        assert!(contract.get_latest_jackpot().is_some());
        assert_eq!(contract.get_jackpots().len(), 1);

        contract.create_jackpot(None, None);
        assert_eq!(contract.get_jackpots().len(), 1);
    }

    #[test]
    fn create_drawed_result() {
        let context = get_context(vec![], false);
        testing_env!(context);
        
        let result = DrawingResult::default();
        let numbers = result.drawed_numbers;

        // let result2 = DrawingResult::default();
        // let numbers2 = result2.drawed_numbers;
        
        println!("Creating a DrawingResult {:?}", numbers);

        let number = get_random_number(MAX_DRAWING_NUMBER.into(), 0);
        println!("Random number is: {}", number);

        assert_eq!(numbers, numbers);
    }

    #[test]
    fn test_compare_numbers() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let number1 = [1, 2, 3, 4, 5, 6];
        let number2 = [1, 2, 3, 4, 5, 6];
        let number3 = [1, 2, 3, 4, 5, 7];
        let number4 = [2, 3, 4, 5, 6, 7];

        assert_eq!(compare_numbers(&number1, &number2), true);
        assert_eq!(compare_numbers(&number1, &number3), false);
        assert_eq!(compare_numbers(&number3, &number4), false);
    }

    #[test]
    fn buy_ticket() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let mut contract = Lottery::new(String::from("bob_near"));

        assert!(contract.get_latest_jackpot().is_none());

        // Create a jackpot
        println!("Create a jackpot");
        contract.create_jackpot(None, None);

        assert!(contract.get_latest_jackpot().is_some());

        // Deposit fund to bob_near
        println!("Deposit fund to bob_near");
        contract.deposit();

        assert_eq!(
            U128::from(DEPOSIT_AMOUNT),
            contract.get_account_balance(&String::from("bob_near"))
        );

        // Buy ticket for bob_near
        println!("Buy ticket for bob_near");
        let expected = [1, 2, 3, 4, 5, 6];
        contract.buy_ticket(expected);
        let latest_jackpot = contract.get_latest_jackpot().unwrap();
        let actual = contract.tickets.get(&1).unwrap();
        assert_eq!(expected, actual.picked_numbers);

        // Check ticket id
        println!("Check ticket id");
        assert_eq!(1, latest_jackpot.ticket_ids.len());
    }

    #[test]
    fn buy_ticket_two_accounts() {
        let account_1 = String::from("bob_near");
        let account_2 = String::from("bob_near_2");
        let deposit_amount_1 = ONE_NEAR * 10;
        let deposit_amount_2 = ONE_NEAR * 20;
        let deposit_amount_1_bought_ticket = ONE_NEAR * 7;
        let deposit_amount_2_bought_ticket = ONE_NEAR * 18;

        let mut context = get_context(vec![], false);
        context.signer_account_id = account_1.clone();
        context.attached_deposit = deposit_amount_1;
        testing_env!(context);

        let mut contract = Lottery::new(String::from("bob_near"));

        assert!(contract.get_latest_jackpot().is_none());

        // Create a jackpot
        println!("Create a jackpot");
        contract.create_jackpot(None, None);

        assert!(contract.get_latest_jackpot().is_some());

        // Deposit fund to bob_near
        println!("Deposit fund to bob_near");
        contract.deposit();

        // Buy ticket for bob_near
        println!("Buy ticket for bob_near");
        let account_1_ticket_1_expected = [1, 2, 3, 4, 5, 6];
        contract.buy_ticket(account_1_ticket_1_expected);

        let account_1_ticket_2_expected = [1, 3, 4, 5, 6, 7];
        contract.buy_ticket(account_1_ticket_2_expected);

        let account_1_ticket_3_expected = [1, 4, 5, 6, 7, 8];
        contract.buy_ticket(account_1_ticket_3_expected);

        // Get mock blockchain context for bob_near_2
        let mut context = get_context(vec![], false);
        context.signer_account_id = account_2.clone();
        context.attached_deposit = deposit_amount_2;
        testing_env!(context);

        // Deposit fund to bob_near_2
        println!("Deposit fund to bob_near_2");
        contract.deposit();

        // Buy ticket for bob_near_2
        println!("Buy ticket for bob_near_2");
        let account_2_ticket_1_expected = [2, 3, 4, 5, 6, 7];
        contract.buy_ticket(account_2_ticket_1_expected);
        
        let account_2_ticket_2_expected = [2, 4, 5, 6, 7, 8];
        contract.buy_ticket(account_2_ticket_2_expected);

        // ---------------- Test balance ----------------
        println!("Test balance of account 1");
        assert_eq!(
            U128::from(deposit_amount_1_bought_ticket),
            //U128::from(deposit_amount_1),
            contract.get_account_balance(&account_1)
        );

        println!("Test balance of account 2");
        assert_eq!(
            U128::from(deposit_amount_2_bought_ticket),
            contract.get_account_balance(&account_2)
        );

        println!("Test balance of account 3 (Not existed)");
        assert_eq!(
            U128::from(0),
            contract.get_account_balance(&String::from("bob_near_3"))
        );

        // Debug account infoes
        println!("---- Account Infoes ----");
        let vec = contract.account_infoes.to_vec();
        for i in 0..vec.len() {
            let (account_id, account_info) = &vec[i];
            println!("Account Id: {}, Balance: {}, Tickets: {:?}", account_id, account_info.balance, account_info.ticket_ids);
            let ticket_ids = account_info.ticket_ids.to_vec();
            for j in 0..ticket_ids.len() {
                println!("--> Ticket id {}: {}", j, ticket_ids[j]);
            }
        }
        println!("---- End of Account Infoes ----");

        // ---------------- Test ticket ----------------
        let tickets = contract.get_account_tickets(&account_1);
        let account_1_ticket_1_actual = tickets.get(0).unwrap().picked_numbers;
        
        println!("Test ticket of account 1");
        assert_eq!(account_1_ticket_1_expected, account_1_ticket_1_actual);
        //assert_eq!(0, tickets.len());

        let tickets = contract.get_account_tickets(&account_2);
        let account_2_ticket_1_actual = tickets.get(0).unwrap().picked_numbers;

        println!("Test ticket of account 2");
        assert_eq!(account_2_ticket_1_expected, account_2_ticket_1_actual);

        // // Check ticket id
        // println!("Check ticket id");
        // assert_eq!(1, latest_jackpot.ticket_ids.len());
    }

    #[test]
    fn test_vector() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let mut vec1 : Vector<u8> = Vector::new(b"v".to_vec());
        let mut vec2 : Vector<u8> = Vector::new(b"v1".to_vec());

        vec1.push(&1);
        vec1.push(&2);

        vec2.push(&3);

        println!("Vector 1: [{}]", vec1.to_vec()[0]);
        println!("Vector 2: [{}]", vec2.to_vec()[0]);
    }

    #[test]
    fn get_random_numbers() {
        let context = get_context(vec![], false);
        testing_env!(context);

        let contract = Lottery::new(String::from("bob_near"));

        assert_eq!(contract.get_number(), 0);
    }
}
