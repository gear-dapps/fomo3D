#![no_std]

use gstd::{exec, msg, prelude::*, ActorId};

mod ft_messages;
use ft_messages::transfer_tokens;

mod fomo3d_io;
use fomo3d_io::*;

#[derive(Debug, Default)]
pub struct Fomo3d {
    token_address: ActorId, // decimals = Fomo3d::PRECISION
    last_user_address: ActorId,
    key_price: u128, // price in ETH (self.token_address)
    keys_sold: u128,
    pot: u128,
    last_update: u64,
    time_left: u64, // default SEC_IN_DAY
}

static mut FOMO3D_CONTRACT: Option<Fomo3d> = None;

impl Fomo3d {
    const PRECISION: u128 = 10_u128.pow(12);
    const SEC_IN_DAY: u64 = 60 * 60 * 24;

    pub async fn buy_key(&mut self) {
        self.update_price();
        self.update_time();

        if self.time_left == 0 {
            self.end_game().await;
            return;
        }

        transfer_tokens(
            &self.token_address,
            &msg::source(),
            &exec::program_id(),
            self.key_price,
        )
        .await;

        self.pot = self
            .pot
            .checked_add(self.key_price)
            .unwrap_or_else(|| panic!("Overflowing add in buy_tokens()"));
        self.keys_sold = self
            .keys_sold
            .checked_add(1)
            .unwrap_or_else(|| panic!("Overflowing add in buy_tokens()"));

        self.last_user_address = msg::source();
        self.last_update = exec::block_timestamp() / 1000;
        self.add_time();

        msg::reply(
            Fomo3dEvent::KeyBought {
                price: self.key_price,
                address: msg::source(),
            },
            0,
        )
        .expect("Error in reply");
    }

    async fn end_game(&mut self) {
        transfer_tokens(
            &self.token_address,
            &exec::program_id(),
            &self.last_user_address,
            self.pot,
        )
        .await;

        self.pot = 0;
    }

    fn time_end(&self) -> bool {
        if self.last_update == 0 {
            return false;
        }

        self.time_left <= self.last_update - exec::block_timestamp() / 1000
    }

    fn add_time(&mut self) {
        if self.time_left + 30 <= Fomo3d::SEC_IN_DAY {
            self.time_left += 30;
            return;
        }

        self.time_left = Fomo3d::SEC_IN_DAY;
    }

    fn update_price(&mut self) {
        if self.keys_sold == 0 {
            self.key_price = Fomo3d::PRECISION;
        }

        let inter = self
            .keys_sold
            .checked_mul(self.keys_sold)
            .unwrap_or_else(|| panic!("Overflowing multiplication in update_price()"));
        self.key_price = inter
            .checked_mul(Fomo3d::PRECISION)
            .unwrap_or_else(|| panic!("Overflowing multiplication in update_price()"));
    }

    fn update_time(&mut self) {
        if self.time_end() {
            self.time_left = 0;
            return;
        }

        self.time_left -= self.last_update - exec::block_timestamp() / 1000;
    }
}

#[gstd::async_main]
async unsafe fn main() {
    let action: Fomo3dAction = msg::load().expect("Unable to decode Fomo3dAction");
    let fomo3d: &mut Fomo3d = unsafe { FOMO3D_CONTRACT.get_or_insert(Default::default()) };

    match action {
        Fomo3dAction::BuyKey => fomo3d.buy_key().await,
    }
}

#[no_mangle]
pub unsafe extern "C" fn init() {
    let config: Fomo3dInit = msg::load().expect("Unable to decode Fomo3dInit");

    let fomo3d = Fomo3d {
        token_address: config.token_address,
        ..Default::default()
    };

    FOMO3D_CONTRACT = Some(fomo3d);
}
