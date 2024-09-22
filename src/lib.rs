#![no_std]
use rshooks_api::{_c::INVALID_TXN, *};

#[rustfmt::skip]
static mut TXN: [u8;250] =  [
    /* size,upto */
    /*   3,  0  */ 0x12, 0x00, 0x2F,                                                 /* tt = URITokenBuy */
    /*   5,  3  */ 0x22, 0x80, 0x00, 0x00, 0x00,                                     /* flags = tfCanonical */
    /*   5,  8  */ 0x24, 0x00, 0x00, 0x00, 0x00,                                     /* sequence = 0 */
    /*   6, 13  */ 0x20, 0x1A, 0x00, 0x00, 0x00, 0x00,                               /* first ledger seq */
    /*   6, 19  */ 0x20, 0x1B, 0x00, 0x00, 0x00, 0x00,                               /* last ledger seq */
    /*   8, 25  */ 0x61, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99, 0x99,             /* amount field 9 or 49 bytes */
    /*  34, 33  */ 0x50, 0x24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,     /* hash256 = URITokenID  */
    /*   9, 67  */ 0x68, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,             /* fee      */
    /*  35, 76  */ 0x73, 0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  /* pubkey   */
    /*  22, 111 */ 0x81, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,                                         /* src acc  */
    /* 116, 133 */ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                   0, 0, 0, 0, 0,                                                    /* emit details */
    /*   0, 249 */
];

#[inline]
fn get_txn_field_ptr<const T: usize>(offset: usize) -> &'static mut Buffer<T> {
    unsafe { &mut *(TXN[offset..].as_mut_ptr() as *mut Buffer<T>) }
}

#[no_mangle]
pub extern "C" fn hook(_reserved: u32) -> i64 {
    _g(1, 1);

    let txn = unsafe { &mut TXN[0..] };
    let indexid_out = get_txn_field_ptr::<32>(36);
    let amount_out = get_txn_field_ptr::<8>(26);
    let hook_acc = get_txn_field_ptr::<20>(114);

    trace(b"autotransfer: Called.", b"", DataRepr::AsUTF8).unwrap();

    // HOOK ON: TT
    let tt: i64 = otxn_type();
    if tt != TxnType::URITokenCreateSellOffer as i64 {
        rollback(
            b"autotransfer: HookOn field is incorrectly set.",
            INVALID_TXN.into(),
        );
    }

    let otx_acc: &mut AccountId = &mut [0u8; 20];

    // FILTER ON: ACCOUNT
    if is_txn_outgoing::<20>(hook_acc, otx_acc).unwrap() {
        accept(b"autotransfer: outgoing tx on `Account`.", 0);
    }

    // TXN: PREPARE: URITokenID
    otxn_field(indexid_out, FieldId::URITokenID).unwrap();

    // TXN: PREPARE: Init
    etxn_reserve(1).unwrap();

    // TXN PREPARE: Buy Amount
    let drops: u64 = 0;
    encode_amount(amount_out, drops);

    autofill_txn(txn);

    // TXN: Emit/Send Txn
    let emithash = &mut [0u8; 32];
    emit(emithash, txn).expect(b"autotransfer: Tx emitted failure.");

    accept(b"autotransfer: Tx emitted success.", 0);
}

#[inline]
fn autofill_txn(txn: &mut [u8]) {
    let fls_out = get_txn_field_ptr::<4>(15);
    let lls_out = get_txn_field_ptr::<4>(21);
    let fee_out = get_txn_field_ptr::<8>(69);
    let emit_out = get_txn_field_ptr::<116>(134);

    // TXN PREPARE: FirstLedgerSequence
    let fls = (ledger_seq() as u32) + 1;
    fls_out.copy_from_slice(&fls.to_be_bytes());

    // TXN PREPARE: LastLedgerSequense
    let lls = fls + 4;
    lls_out.copy_from_slice(&lls.to_be_bytes());

    // TXN PREPARE: Emit Metadata
    etxn_details(emit_out).unwrap();

    // TXN PREPARE: Fee
    let fee = etxn_fee_base(txn).expect(b"autotransfer: Failed to get fee.");
    encode_amount(fee_out, fee as u64);
}

#[inline]
fn encode_amount(out: &mut NativeAmount, amount: u64) {
    out[0] = 0b01000000 + ((amount >> 56) & 0b00111111) as u8;
    out[1] = ((amount >> 48) & 0xFF) as u8;
    out[2] = ((amount >> 40) & 0xFF) as u8;
    out[3] = ((amount >> 32) & 0xFF) as u8;
    out[4] = ((amount >> 24) & 0xFF) as u8;
    out[5] = ((amount >> 16) & 0xFF) as u8;
    out[6] = ((amount >> 8) & 0xFF) as u8;
    out[7] = ((amount >> 0) & 0xFF) as u8;
}
