use crate::liqudation::users::User;


//deposit
//withdraw
//market_order
//limit_order
//cancel_order
//transfer
//liquidation
//borrow


pub struct Deposit {
    user_id: u128,
    amount: [u8;32],
}


pub fn deposit(_deposit: Deposit, _user: &mut User) {

}


