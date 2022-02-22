
use crate::*;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct AccountInfo {
    pub balance: Balance,
    // TODO: Should be defined as reference type
    pub ticket_ids: Vector<TicketId>,
    created_time: Timestamp
}

impl Serialize for AccountInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AccountInfo", 3)?;
        state.serialize_field("balance", &self.balance.to_string())?;
        state.serialize_field("ticketIds", &self.ticket_ids.to_vec())?;
        state.serialize_field("createdTime", &self.created_time)?;
        state.end()
    }
}

impl AccountInfo {
    pub fn new(key: AccountId) -> Self {
        Self {
            balance: 0,
            ticket_ids: Vector::new(format!("ta{}", key).as_bytes()),
            created_time: get_time_now(),
        }
    }
}