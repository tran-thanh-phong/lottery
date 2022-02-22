use crate::*;


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Ticket {
    id: TicketId,
    pub account_id: AccountId,   
    pub picked_numbers: [u8; 6],
    created_time: Timestamp
}

impl Serialize for Ticket {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Ticket", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("accountId", &self.account_id)?;
        state.serialize_field("pickedNumbers", &self.picked_numbers)?;
        state.serialize_field("createdTime", &self.created_time)?;
        state.end()
    }
}

impl Ticket {
    pub fn new(id: &TicketId, account_id: &AccountId, picked_numbers: &[u8; 6]) -> Self {
        Self {
            id: *id,
            account_id: account_id.clone(),
            picked_numbers: *picked_numbers,
            created_time: get_time_now(),
        }
    }
}
