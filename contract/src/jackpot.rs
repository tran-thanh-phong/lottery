use crate::*;

pub enum JackpotStatus {
    Open,
    Close
}

impl Serialize for JackpotStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            JackpotStatus::Open => serializer.serialize_unit_variant("JackpotStatus", 0, "Open"),
            JackpotStatus::Close => serializer.serialize_unit_variant("JackpotStatus", 1, "Close"),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Jackpot {
    id: JackpotId,
    pub ticket_price: u128,
    pub locked_amount: Balance,
    pub ticket_ids: Vector<TicketId>,
    pub win_ticket_ids: Vector<TicketId>,
    pub drawed_results: Vector<DrawingResult>,
    start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    created_time: Timestamp,
}

impl Jackpot {
    pub fn new(id: u32, start_time: Timestamp, ticket_price: u128, initialized_amount: u128) -> Self {
        Self {
            id,
            ticket_price,
            locked_amount: initialized_amount,
            ticket_ids: Vector::new(format!("tj{}", id).as_bytes()),
            win_ticket_ids: Vector::new(format!("tjw{}", id).as_bytes()),
            drawed_results: Vector::new(format!("dr{}", id).as_bytes()),
            start_time,
            end_time: Option::None,
            created_time: get_time_now(),
        }
    }

    pub fn get_status(&self) -> JackpotStatus {
        let now = get_time_now();
        match self.end_time {
            None => {
                if self.start_time <= now {
                    JackpotStatus::Open
                }
                else {
                    JackpotStatus::Close
                }
            },
            Some(t) => {
                if self.start_time <= now && now <= t {
                    JackpotStatus::Open
                }
                else {
                    JackpotStatus::Close
                }
            },
        }
    }
}

impl Serialize for Jackpot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Jackpot", 1)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("ticketPrice", &self.ticket_price.to_string())?;
        state.serialize_field("lockedAmount", &self.locked_amount.to_string())?;
        state.serialize_field("startTime", &self.start_time)?;
        state.serialize_field("createdTime", &self.created_time)?;
        state.serialize_field("endTime", &self.end_time)?;
        state.serialize_field("status", &self.get_status())?;
        
        state.serialize_field("noOfTickets", &self.ticket_ids.len())?;
        state.serialize_field("ticketIds", &self.ticket_ids.to_vec())?;
        state.serialize_field("winTicketIds", &self.win_ticket_ids.to_vec())?;
        state.serialize_field("drawedResults", &self.drawed_results.to_vec())?;

        state.end()
    }
}
