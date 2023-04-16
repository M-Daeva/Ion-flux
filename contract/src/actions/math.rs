use std::ops::Div;

use cosmwasm_std::{Decimal, StdError, StdResult, Timestamp, Uint128};

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

// interpolation for linear function y(t), t in range t1...t2, y in range y1...y2
// y(t) = y1 + (y2 - y1) * (t - t1) / (t2 - t1)
fn interpolate(
    y1: Uint128,
    y2: Uint128,
    t1: Timestamp,
    t2: Timestamp,
    t: Timestamp,
) -> StdResult<Uint128> {
    let t1 = t1.nanos();
    let t2 = t2.nanos();
    let t = t.nanos();

    if t2 <= t1 {
        Err(StdError::generic_err("t2 <= t1 at interpolate"))?
    }

    if t < t1 {
        Err(StdError::generic_err("t < t1 at interpolate"))?
    }

    if (y2 == y1) || (t == t1) {
        return Ok(y1);
    }

    let y = if y2 > y1 {
        y1 + (y2 - y1) * Uint128::from(t - t1) / Uint128::from(t2 - t1)
    } else {
        y1 - (y1 - y2) * Uint128::from(t - t1) / Uint128::from(t2 - t1)
    };

    Ok(y)
}

// area under line y(t), t in range t1...t2, y in range y1...y2
// a = (y2 + y1) * (t2 - t1) / 2
fn calc_area(y1: Uint128, y2: Uint128, t1: Timestamp, t2: Timestamp) -> StdResult<Uint128> {
    let t1 = t1.nanos();
    let t2 = t2.nanos();

    if t2 < t1 {
        Err(StdError::generic_err("t2 < t1 at calc_area"))?
    }

    if t2 == t1 {
        return Ok(Uint128::zero());
    }

    let a = ((y2 + y1) * Uint128::from(t2 - t1)).div(Uint128::from(2_u128));

    Ok(a)
}

// removes from vector values older <last_element_timestamp - window>
// also applies linear interpolation to get a point on left boundary
fn frame_list(sample_list: &Vec<Sample>, window: Uint128) -> StdResult<Vec<Sample>> {
    // check if sample list isn't empty
    if sample_list.is_empty() {
        Err(StdError::generic_err("sample list is empty at frame_list"))?;
    }

    // let mut boundary_value = Uint128::default();
    let mut framed_list: Vec<Sample> = vec![];

    // at least 1 sample is required to get boundary sample
    let boundary_sample = sample_list
        .last()
        .ok_or_else(|| StdError::generic_err("boundary_timestamp reading error at frame_list"))?;

    if boundary_sample.timestamp < Timestamp::from_nanos(window.u128() as u64) {
        Err(StdError::generic_err("window is too large at frame_list"))?;
    }

    let boundary_timestamp = boundary_sample.timestamp.minus_nanos(window.u128() as u64);

    // theoretically boundary_left < boundary_saple < boundary_right for timestamps of sample list
    let mut boundary_left = sample_list
        .first()
        .ok_or_else(|| StdError::generic_err("boundary_left reading error at frame_list"))?
        .clone();

    // if boundary_left.timestamp > boundary_timestamp it means all samples are inside window
    // then sample with zero value must be added as boundary sample
    if boundary_left.timestamp > boundary_timestamp {
        return Ok([
            vec![Sample::new(Uint128::zero(), boundary_timestamp)],
            sample_list.to_vec(),
        ]
        .concat());
    }

    // if boundary_left.timestamp == boundary_right.timestamp we already have boundary_value
    if boundary_left.timestamp == boundary_timestamp {
        return Ok(sample_list.to_vec());
    }

    for sample in sample_list {
        if sample.timestamp < boundary_timestamp {
            boundary_left = sample.clone();
        } else {
            framed_list.push(sample.clone());
        }
    }

    let boundary_right = framed_list
        .first()
        .ok_or_else(|| StdError::generic_err("boundary_right reading error at frame_list"))?;

    let boundary_value = interpolate(
        boundary_left.value,
        boundary_right.value,
        boundary_left.timestamp,
        boundary_right.timestamp,
        boundary_timestamp,
    )?;

    let framed_list = [
        vec![Sample::new(boundary_value, boundary_timestamp)],
        framed_list,
    ]
    .concat();

    Ok(framed_list)
}

// returns average value of zigzag function on its window range
fn calc_average(sample_list: &Vec<Sample>) -> StdResult<Uint128> {
    // check if sample list isn't empty
    if sample_list.is_empty() {
        Err(StdError::generic_err(
            "sample list is empty at calc_average",
        ))?;
    }

    let mut area_acc = Uint128::zero();
    let mut sample_pre = sample_list
        .first()
        .ok_or_else(|| StdError::generic_err("sample_pre reading error at calc_average"))?;

    if sample_list.len() == 1 {
        return Ok(sample_pre.value);
    }

    for sample in sample_list {
        let area = calc_area(
            sample_pre.value,
            sample.value,
            sample_pre.timestamp,
            sample.timestamp,
        )?;

        area_acc += area;
        sample_pre = sample;
    }

    let timestamp_last = sample_list
        .last()
        .ok_or_else(|| StdError::generic_err("timestamp_last reading error at calc_average"))?
        .timestamp
        .nanos();
    let timestamp_first = sample_list
        .first()
        .ok_or_else(|| StdError::generic_err("timestamp_first reading error at calc_average"))?
        .timestamp
        .nanos();

    if timestamp_last < timestamp_first {
        Err(StdError::generic_err(
            "timestamp_last < timestamp_first at calc_average",
        ))?
    }

    let window = Uint128::from(timestamp_last - timestamp_first);

    Ok(area_acc / window)
}

pub fn calc_sma(
    sample_list: &Vec<Sample>,
    sample: &Sample,
    window: Uint128,
) -> StdResult<(Vec<Sample>, Uint128)> {
    let updated_list = [sample_list.to_owned(), vec![sample.to_owned()]].concat();
    let framed_list = frame_list(&updated_list, window)?;
    let sma = calc_average(&framed_list)?;

    Ok((framed_list, sma))
}

// volume_ratio = (unbonded + requested + swapped_out) / (bonded + (1 - swap_fee_rate) * swapped_in)
// all values used as SMA
pub fn calc_volume_ratio(
    bonded: Uint128,
    unbonded: Uint128,
    requested: Uint128,
    swapped_in: Uint128,
    swapped_out: Uint128,
    swap_fee_rate: Decimal,
) -> StdResult<Decimal> {
    const MAX_RATIO: u128 = 100;

    let max_ratio = u128_to_dec(MAX_RATIO);
    let one = Decimal::one();

    if swap_fee_rate >= one {
        Err(StdError::generic_err(
            "swap_fee_rate >= one at calc_volume_ratio",
        ))?
    }

    let volume_in = uint128_to_dec(bonded) + (one - swap_fee_rate) * uint128_to_dec(swapped_in);
    let volume_out = uint128_to_dec(unbonded + requested + swapped_out);

    if !volume_in.is_zero() && !volume_out.is_zero() {
        Ok((volume_out / volume_in).clamp(one / max_ratio, max_ratio))
    } else if !volume_out.is_zero() {
        Ok(max_ratio)
    } else {
        Ok(one / max_ratio)
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

#[cfg(test)]
pub mod test {
    use super::{
        calc_area, calc_average, calc_sma, calc_volume_ratio, frame_list, interpolate, str_to_dec,
        Sample,
    };
    use cosmwasm_std::{StdError, Timestamp, Uint128};

    #[test]
    fn interpolate_up() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let y = Uint128::from(1500u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let t = Timestamp::from_nanos(150);

        assert_eq!(interpolate(y1, y2, t1, t2, t).unwrap(), y);
    }

    #[test]
    fn interpolate_down() {
        let y1 = Uint128::from(2000u128);
        let y2 = Uint128::from(1000u128);
        let y = Uint128::from(1500u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let t = Timestamp::from_nanos(150);

        assert_eq!(interpolate(y1, y2, t1, t2, t).unwrap(), y);
    }

    #[test]
    fn interpolate_div_by_zero() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(100);
        let t = Timestamp::from_nanos(100);

        assert_eq!(
            interpolate(y1, y2, t1, t2, t).unwrap_err(),
            StdError::generic_err("t2 <= t1 at interpolate")
        );
    }

    #[test]
    fn interpolate_t_is_smaller_t1() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let t = Timestamp::from_nanos(90);

        assert_eq!(
            interpolate(y1, y2, t1, t2, t).unwrap_err(),
            StdError::generic_err("t < t1 at interpolate")
        );
    }

    #[test]
    fn interpolate_y2_is_equal_y1() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(1000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let t = Timestamp::from_nanos(150);

        assert_eq!(interpolate(y1, y2, t1, t2, t).unwrap(), y1);
    }

    #[test]
    fn interpolate_t_is_equal_t1() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let t = Timestamp::from_nanos(100);

        assert_eq!(interpolate(y1, y2, t1, t2, t).unwrap(), y1);
    }

    #[test]
    fn calc_area_up() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let a = Uint128::from(150_000u128);

        assert_eq!(calc_area(y1, y2, t1, t2).unwrap(), a);
    }

    #[test]
    fn calc_area_down() {
        let y1 = Uint128::from(2000u128);
        let y2 = Uint128::from(1000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(200);
        let a = Uint128::from(150_000u128);

        assert_eq!(calc_area(y1, y2, t1, t2).unwrap(), a);
    }

    #[test]
    fn calc_area_t2_is_smaller_t1() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let t1 = Timestamp::from_nanos(200);
        let t2 = Timestamp::from_nanos(100);

        assert_eq!(
            calc_area(y1, y2, t1, t2).unwrap_err(),
            StdError::generic_err("t2 < t1 at calc_area")
        );
    }

    #[test]
    fn calc_area_t2_is_equal_t1() {
        let y1 = Uint128::from(1000u128);
        let y2 = Uint128::from(2000u128);
        let t1 = Timestamp::from_nanos(100);
        let t2 = Timestamp::from_nanos(100);
        let a = Uint128::zero();

        assert_eq!(calc_area(y1, y2, t1, t2).unwrap(), a);
    }

    #[test]
    fn frame_list_samples_outside_boundary() {
        let sample_list = vec![
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(0)),
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];
        let window = Uint128::from(1500u128);
        let framed_list = vec![
            Sample::new(Uint128::from(1500u128), Timestamp::from_nanos(1500)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];

        assert_eq!(frame_list(&sample_list, window).unwrap(), framed_list);
    }

    #[test]
    fn frame_list_sample_on_boundary() {
        let sample_list = vec![
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1500)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];
        let window = Uint128::from(1500u128);
        let framed_list = vec![
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1500)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];

        assert_eq!(frame_list(&sample_list, window).unwrap(), framed_list);
    }

    #[test]
    fn frame_list_samples_inside_window() {
        let sample_list = vec![
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];
        let window = Uint128::from(2500u128);
        let framed_list = vec![
            Sample::new(Uint128::zero(), Timestamp::from_nanos(500)),
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];

        assert_eq!(frame_list(&sample_list, window).unwrap(), framed_list);
    }

    #[test]
    fn frame_list_window_is_too_large() {
        let sample_list = vec![
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
        ];
        let window = Uint128::from(3500u128);

        assert_eq!(
            frame_list(&sample_list, window).unwrap_err(),
            StdError::generic_err("window is too large at frame_list")
        );
    }

    #[test]
    fn frame_list_empty() {
        let sample_list = vec![];
        let window = Uint128::from(1500u128);

        assert_eq!(
            frame_list(&sample_list, window).unwrap_err(),
            StdError::generic_err("sample list is empty at frame_list")
        );
    }

    #[test]
    fn frame_list_single_sample() {
        let sample_list = vec![Sample::new(
            Uint128::from(1000u128),
            Timestamp::from_nanos(1000),
        )];
        let window = Uint128::from(500u128);
        let framed_list = vec![
            Sample::new(Uint128::zero(), Timestamp::from_nanos(500)),
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
        ];

        assert_eq!(frame_list(&sample_list, window).unwrap(), framed_list);
    }

    #[test]
    fn calc_average_empty() {
        let sample_list = vec![];

        assert_eq!(
            calc_average(&sample_list).unwrap_err(),
            StdError::generic_err("sample list is empty at calc_average")
        );
    }

    #[test]
    fn calc_average_single_sample() {
        let sample_list = vec![Sample::new(
            Uint128::from(1000u128),
            Timestamp::from_nanos(1000),
        )];
        let average = Uint128::from(1000u128);

        assert_eq!(calc_average(&sample_list).unwrap(), average);
    }

    #[test]
    fn calc_average_2_samples() {
        let sample_list = vec![
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(3000)),
        ];
        let average = Uint128::from(2500u128);

        assert_eq!(calc_average(&sample_list).unwrap(), average);
    }

    #[test]
    fn calc_average_4_samples() {
        let sample_list = vec![
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(0)),
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
            Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(3000)),
        ];
        let average = Uint128::from(2000u128);

        assert_eq!(calc_average(&sample_list).unwrap(), average);
    }

    #[test]
    fn calc_average_4_samples_unequal() {
        let sample_list = vec![
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(0)),
            Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
            Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(3000)),
            Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(6000)),
        ];
        let average = Uint128::from(2250u128);

        assert_eq!(calc_average(&sample_list).unwrap(), average);
    }

    #[test]
    fn calc_sma_empty() {
        let sample_list = vec![];
        let sample = Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000));
        let window = Uint128::from(1000u128);
        let average = Uint128::from(500u128);

        let (updated_list, sma) = calc_sma(&sample_list, &sample, window).unwrap();

        assert_eq!(
            updated_list,
            vec![
                Sample::new(Uint128::zero(), Timestamp::from_nanos(0)),
                sample
            ]
        );
        assert_eq!(sma, average);
    }

    #[test]
    fn calc_sma_single_asset() {
        let sample_list = vec![Sample::new(
            Uint128::from(1000u128),
            Timestamp::from_nanos(1000),
        )];
        let sample = Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000));
        let window = Uint128::from(1000u128);
        let average = Uint128::from(1500u128);

        let (_updated_list, sma) = calc_sma(&sample_list, &sample, window).unwrap();

        assert_eq!(sma, average);
    }

    #[test]
    fn calc_sma_different_windows() {
        // equal distance, window 1k
        assert_eq!(
            calc_sma(
                &vec![
                    Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
                    Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
                    Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
                ],
                &Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(4000)),
                Uint128::from(1000u128)
            )
            .unwrap()
            .1,
            Uint128::from(3500u128)
        );

        // equal distance, window 2k
        assert_eq!(
            calc_sma(
                &vec![
                    Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
                    Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
                    Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
                ],
                &Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(4000)),
                Uint128::from(2000u128)
            )
            .unwrap()
            .1,
            Uint128::from(3000u128)
        );

        // equal distance, window 3k
        assert_eq!(
            calc_sma(
                &vec![
                    Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
                    Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
                    Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
                ],
                &Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(4000)),
                Uint128::from(3000u128)
            )
            .unwrap()
            .1,
            Uint128::from(2500u128)
        );

        // equal distance, window 3.5k
        assert_eq!(
            calc_sma(
                &vec![
                    Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
                    Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
                    Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
                ],
                &Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(4000)),
                Uint128::from(3500u128)
            )
            .unwrap()
            .1,
            Uint128::from(2214u128)
        );

        // equal distance, window 4k
        assert_eq!(
            calc_sma(
                &vec![
                    Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
                    Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
                    Sample::new(Uint128::from(3000u128), Timestamp::from_nanos(3000)),
                ],
                &Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(4000)),
                Uint128::from(4000u128)
            )
            .unwrap()
            .1,
            Uint128::from(2000u128)
        );

        // unequal distance, window 2k
        assert_eq!(
            calc_sma(
                &vec![
                    Sample::new(Uint128::from(1000u128), Timestamp::from_nanos(1000)),
                    Sample::new(Uint128::from(2000u128), Timestamp::from_nanos(2000)),
                ],
                &Sample::new(Uint128::from(4000u128), Timestamp::from_nanos(4000)),
                Uint128::from(2000u128)
            )
            .unwrap()
            .1,
            Uint128::from(3000u128)
        );
    }

    #[test]
    fn calc_volume_ratio_default() {
        let bonded = Uint128::from(1000u128);
        let unbonded = Uint128::from(500u128);
        let requested = Uint128::from(500u128);
        let swapped_in = Uint128::from(1000u128);
        let swapped_out = Uint128::from(1000u128);
        let swap_fee_rate = str_to_dec("0.003");

        let res = calc_volume_ratio(
            bonded,
            unbonded,
            requested,
            swapped_in,
            swapped_out,
            swap_fee_rate,
        );

        let volume_ratio = str_to_dec("1.001502253380070105");

        assert_eq!(res.unwrap(), volume_ratio);
    }

    #[test]
    fn calc_volume_ratio_fee_is_too_large() {
        let bonded = Uint128::from(1000u128);
        let unbonded = Uint128::from(500u128);
        let requested = Uint128::from(500u128);
        let swapped_in = Uint128::from(1000u128);
        let swapped_out = Uint128::from(1000u128);
        let swap_fee_rate = str_to_dec("1");

        let res = calc_volume_ratio(
            bonded,
            unbonded,
            requested,
            swapped_in,
            swapped_out,
            swap_fee_rate,
        );

        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("swap_fee_rate >= one at calc_volume_ratio")
        );
    }

    #[test]
    fn calc_volume_ratio_limit_lower() {
        let bonded = Uint128::from(100_000_000u128);
        let unbonded = Uint128::one();
        let requested = Uint128::one();
        let swapped_in = Uint128::one();
        let swapped_out = Uint128::one();
        let swap_fee_rate = str_to_dec("0.003");

        let res = calc_volume_ratio(
            bonded,
            unbonded,
            requested,
            swapped_in,
            swapped_out,
            swap_fee_rate,
        );

        let volume_ratio = str_to_dec("0.01");

        assert_eq!(res.unwrap(), volume_ratio);
    }

    #[test]
    fn calc_volume_ratio_limit_upper() {
        let bonded = Uint128::one();
        let unbonded = Uint128::one();
        let requested = Uint128::from(100_000_000u128);
        let swapped_in = Uint128::one();
        let swapped_out = Uint128::one();
        let swap_fee_rate = str_to_dec("0.003");

        let res = calc_volume_ratio(
            bonded,
            unbonded,
            requested,
            swapped_in,
            swapped_out,
            swap_fee_rate,
        );

        let volume_ratio = str_to_dec("100");

        assert_eq!(res.unwrap(), volume_ratio);
    }
}
