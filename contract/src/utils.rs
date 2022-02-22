use crate::*;


pub(crate) fn get_random_number(max: u64, mut ran_no: u64) -> u64 {
    let mut random_number = 0; // *env::random_seed().get(0).unwrap();

    // TODO: This is used for testing, try using #DEBUG
    if random_number == 0 {
        let mut time_value = get_time_now();
        while time_value % 10 == 0 {
            time_value /= 10;
        }

        if ran_no <= 0 {
            ran_no = 1;
        }

        random_number = (time_value + ran_no) / (113 + ran_no) * (77 + ran_no) % max;
    }

    random_number % max + 1
}

pub(crate) fn get_time_now() -> Timestamp {
    env::block_timestamp()
}

pub(crate) fn compare_numbers(n1: &[u8; 6], n2: &[u8; 6]) -> bool {
    for i in [0..6] {
        if n1[i.clone()] != n2[i] { 
            return false;
        }
    }

    true
}