#![no_std]
use rshooks_api::{
    _c::{ttURITOKEN_CREATE_SELL_OFFER, INVALID_TXN},
    *,
};

#[rustfmt::skip]
static mut TXN: [u8;250] =  [
    /* size,upto */
    /*   3,  0  */ 0x12, 0x00, 0x2F,                                                 /* tt = URITokenBuy */
    /*   5,  3  */ 0x22, 0x80, 0x00, 0x00, 0x00,                                     /* flags = tfCanonical */
    /*   5,  8  */ 0x24, 0x00, 0x00, 0x00, 0x00,                                     /* sequence = 0 */
    /*   6, 13  */ 0x20, 0x1A, 0x00, 0x00, 0x00, 0x00,                               /* first ledger seq */
    /*   6, 19  */ 0x20, 0x1B, 0x00, 0x00, 0x00, 0x00,                               /* last ledger seq */
    /*  8, 25  */ 0x61, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99,              /* amount field 9 or 49 bytes */
    
    /*  34, 33  */ 0x50, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,    /* hash256 = URITokenID  */
    /*   9, 67 */ 0x68, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,                   /* fee      */
    0x00,
    /*  35, 76 */ 0x73, 0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* pubkey   */
    /*  22, 111 */ 0x81, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,                                        /* src acc  */
    /* 116, 133 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* emit details */
    /*   0, 249 */
];

// TX BUILDER

#[no_mangle]
pub extern "C" fn hook(_reserved: u32) -> i64 {
    _g(1, 1);

    let FLS_OUT = unsafe { &mut TXN[15..19] };
    let LLS_OUT = unsafe { &mut TXN[21..25] };
    let INDEXID_OUT = unsafe { &mut TXN[36..68] };
    let AMOUNT_OUT = unsafe { &mut TXN[25..34] };
    let HOOK_ACC = unsafe { &mut TXN[114..134] };
    let FEE_OUT = unsafe { &mut TXN[69..77] };
    let EMIT_OUT = unsafe { &mut TXN[134..250] };

    let _ = trace(b"autotransfer.c: Called.", b"", DataRepr::AsUTF8);

    // HOOK ON: TT
    let tt = otxn_type();
    if tt != ttURITOKEN_CREATE_SELL_OFFER.into() {
        rollback(
            b"autotransfer.c: HookOn field is incorrectly set.",
            INVALID_TXN.into(),
        );
    }

    // ACCOUNT: Hook Account
    let _ = hook_account(&mut HOOK_ACC[0..]);

    // ACCOUNT: Origin Tx Account
    let mut otx_acc = [0u8; 20];
    let _ = otxn_field(&mut otx_acc, FieldId::Account);

    // FILTER ON: ACCOUNT
    if is_buffer_equal::<20>(&HOOK_ACC, &otx_acc) {
        accept(b"autotransfer.c: outgoing tx on `Account`.", 0);
    }

    let mut tid_buffer = [0u8; 32];
    match otxn_field(&mut tid_buffer, FieldId::URITokenID) {
        Ok(32) => (),
        _ => {
            rollback(b"autotransfer.c: Invalid URITokenID", INVALID_TXN.into());
        }
    }
    INDEXID_OUT.copy_from_slice(&tid_buffer);

    // TXN: PREPARE: Init
    let _ = etxn_reserve(1);

    // TXN PREPARE: FirstLedgerSequence
    let fls = (ledger_seq() as u32) + 1;
    FLS_OUT.copy_from_slice(&fls.to_be_bytes());

    // TXN PREPARE: LastLedgerSequense
    let lls = fls + 4;
    LLS_OUT.copy_from_slice(&lls.to_be_bytes());

    // // TXN PREPARE: Amount
    let drops: u64 = 0;
    {
        let b = &mut AMOUNT_OUT[1..];
        b[0] = 0b01000000 + ((drops >> 56) & 0b00111111) as u8;
        b[1] = ((drops >> 48) & 0xFF) as u8;
        b[2] = ((drops >> 40) & 0xFF) as u8;
        b[3] = ((drops >> 32) & 0xFF) as u8;
        b[4] = ((drops >> 24) & 0xFF) as u8;
        b[5] = ((drops >> 16) & 0xFF) as u8;
        b[6] = ((drops >> 8) & 0xFF) as u8;
        b[7] = (drops & 0xFF) as u8;
    }

    // TXN PREPARE: Emit Metadata
    let _ = etxn_details(&mut EMIT_OUT[0..]);

    // // TXN PREPARE: Fee
    let fee = match etxn_fee_base(unsafe { &TXN[0..] }) {
        Ok(fee) => fee,
        Err(e) => {
            rollback(b"autotransfer.c: Failed to get fee.", e.code() as i64);
        }
    };

    {
        let b = &mut FEE_OUT[0..];
        b[0] = 0b01000000 + ((fee >> 56) & 0b00111111) as u8;
        b[1] = ((fee >> 48) & 0xFF) as u8;
        b[2] = ((fee >> 40) & 0xFF) as u8;
        b[3] = ((fee >> 32) & 0xFF) as u8;
        b[4] = ((fee >> 24) & 0xFF) as u8;
        b[5] = ((fee >> 16) & 0xFF) as u8;
        b[6] = ((fee >> 8) & 0xFF) as u8;
        b[7] = (fee & 0xFF) as u8;
    }

    // TXN: Emit/Send Txn
    let mut emithash = [0u8; 32];
    let emit_result = match emit(&mut emithash, unsafe { &mut TXN[0..] }) {
        Ok(result) => result,
        Err(_) => {
            rollback(b"autotransfer: Failed to emit tx.", INVALID_TXN.into());
        }
    };

    if emit_result > 0 {
        accept(b"autotransfer: Tx emitted success.", 0);
    }
    accept(b"autotransfer: Tx emitted failure.", 0);
}
