//! A hook template

#![no_std]

use rshooks_api::*;

const GUARD_ID_1: u32 = line!();

#[no_mangle]
pub extern "C" fn cbak(_: u32) -> i64 {
    return 0;
}

#[no_mangle]
#[allow(unused_must_use)]
pub extern "C" fn hook(_: i64) -> i64 {
    // Every hook needs to import guard function
    // and use it at least once
    _g(GUARD_ID_1, 1);

    let mut account: AccountId = uninit_buf!();
    hook_account(&mut account);

    let mut raddr: AccountId = uninit_buf!();
    util_raddr(&mut raddr, &account);

    // Tracing when compiling in debug mode
    #[cfg(debug_assertions)]
    trace(b"Accept: called", &raddr, DataRepr::AsUTF8);

    // Accept all
    accept(b"Accept: accepted", 0)
}
