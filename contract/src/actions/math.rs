use std::ops::Div;

use cosmwasm_std::{Decimal, Timestamp, Uint128};

use crate::state::Sample;

pub fn str_to_dec(s: &str) -> Decimal {
    s.to_string().parse::<Decimal>().unwrap()
}

pub fn str_vec_to_dec_vec(str_vec: Vec<&str>) -> Vec<Decimal> {
    str_vec.iter().map(|&x| str_to_dec(x)).collect()
}

pub fn u128_to_dec(num: u128) -> Decimal {
    Decimal::from_ratio(Uint128::new(num), Uint128::one())
}

pub fn dec_to_u128(dec: Decimal) -> u128 {
    dec.ceil().atomics().u128() / 1_000_000_000_000_000_000
}

pub fn uint128_to_dec(num: Uint128) -> Decimal {
    Decimal::from_ratio(num, Uint128::one())
}

pub fn dec_to_uint128(dec: Decimal) -> Uint128 {
    dec.ceil()
        .atomics()
        .div(Uint128::from(1_000_000_000_000_000_000_u128))
}

pub fn u128_vec_to_uint128_vec(u128_vec: Vec<u128>) -> Vec<Uint128> {
    u128_vec
        .iter()
        .map(|&x| Uint128::from(x as u128))
        .collect::<Vec<Uint128>>()
}

// linear interpolation for linear function y(t), t in range t1...t2, y in range y1...y2
fn interpolate(y1: Uint128, y2: Uint128, t1: Timestamp, t2: Timestamp, t: Timestamp) -> Uint128 {
    let t1 = t1.nanos();
    let t2 = t2.nanos();
    let t = t.nanos();

    y1 + (y2 - y1) * Uint128::from((t - t1) / (t2 - t1))
}

// area under line y(t), t in range t1...t2, y in range y1...y2
fn calc_area(y1: Uint128, y2: Uint128, t1: Timestamp, t2: Timestamp) -> Uint128 {
    let t1 = t1.nanos();
    let t2 = t2.nanos();

    ((y2 + y1) * Uint128::from(t2 - t1)).div(Uint128::from(2_u128))
}

// removes from vector values older <last_element_timestamp - window>
// also applies linear interpolation to get a point on left boundary
fn frame_list(sample_list: &Vec<Sample>, window: Uint128) -> Vec<Sample> {
    // check if sample list isn't empty to use unwrap() on first() and last()
    if sample_list.is_empty() {
        panic!("Sample list is empty!");
    }

    let boundary_timestamp = sample_list
        .last()
        .unwrap()
        .timestamp
        .minus_nanos(window.u128() as u64);
    let mut boundary_left = sample_list.first().unwrap().clone();
    let mut framed_list: Vec<Sample> = vec![];

    for sample in sample_list {
        if sample.timestamp < boundary_timestamp {
            boundary_left = sample.clone();
        } else {
            framed_list.push(sample.clone());
        }
    }

    let boundary_right = framed_list.first().unwrap();
    let boundary_value = interpolate(
        boundary_left.value,
        boundary_right.value,
        boundary_left.timestamp,
        boundary_right.timestamp,
        boundary_timestamp,
    );

    [
        vec![Sample::new(boundary_value, boundary_timestamp)],
        framed_list,
    ]
    .concat()
}

// returns average value of zigzag function on its window range
fn calc_average(sample_list: &Vec<Sample>) -> Uint128 {
    // check if sample list isn't empty to use unwrap() on first() and last()
    if sample_list.is_empty() {
        panic!("Sample list is empty!");
    }

    let mut area_acc = Uint128::zero();
    let mut sample_pre = sample_list.first().unwrap();

    for sample in sample_list {
        let area = calc_area(
            sample_pre.value,
            sample.value,
            sample_pre.timestamp,
            sample.timestamp,
        );

        area_acc += area;
        sample_pre = sample;
    }

    let timestamp_last = sample_list.last().unwrap().timestamp.nanos();
    let timestamp_first = sample_list.first().unwrap().timestamp.nanos();
    let window = Uint128::from(timestamp_last - timestamp_first);

    area_acc / window
}

pub fn calc_sma(
    sample_list: &Vec<Sample>,
    sample: &Sample,
    window: Uint128,
) -> (Vec<Sample>, Uint128) {
    let updated_list = [sample_list.to_owned(), vec![sample.to_owned()]].concat();
    let framed_list = frame_list(&updated_list, window);
    let sma = calc_average(&framed_list);
    (framed_list, sma)
}

// TODO: add tests
// volume_ratio = (unbonded + requested + swapped_out) / (bonded + (1 - swap_fee_rate) * swapped_in)
// all values used as SMA
pub fn calc_volume_ratio(
    bonded: Uint128,
    unbonded: Uint128,
    requested: Uint128,
    swapped_in: Uint128,
    swapped_out: Uint128,
    swap_fee_rate: Decimal,
) -> Decimal {
    const MAX_RATIO: u128 = 100;

    let max_ratio = u128_to_dec(MAX_RATIO);
    let one = Decimal::one();

    let volume_in = uint128_to_dec(bonded) + (one - swap_fee_rate) * uint128_to_dec(swapped_in);
    let volume_out = uint128_to_dec(unbonded + requested + swapped_out);

    if !volume_in.is_zero() && !volume_out.is_zero() {
        volume_out / volume_in
    } else if !volume_out.is_zero() {
        max_ratio
    } else {
        one / max_ratio
    }
}

// TODO: make it actual function
// pub fn calc_provider_reward() {
//     let token_weight = calc_volume_ratio(bonded, unbonded, requested, swapped_in, swapped_out, swap_fee_rate);
//     let allocation = token_amount_bonded_by_one_provider / token_amount_bonded_by_all_providers;
//     let provider_weight = token_weight * allocation;
//     let swap_fee = swap_fee_rate * swapped_in;
//     let provider_reward = swap_fee * provider_weight;
// }
