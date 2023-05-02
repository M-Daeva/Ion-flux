use std::ops::Div;

use cosmwasm_std::{Addr, Decimal, StdError, StdResult, Timestamp, Uint128};

use crate::state::{Asset, Sample, Token};

pub fn str_to_dec(s: &str) -> Decimal {
    s.to_string().parse::<Decimal>().unwrap()
}

pub fn u128_to_dec<T: Into<Uint128>>(num: T) -> Decimal {
    Decimal::from_ratio(num.into(), Uint128::one())
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

// volume_ratio = (requested + swapped_out) / (bonded + (1 - swap_fee_rate) * swapped_in)
// all values used as SMA
pub fn calc_volume_ratio(
    bonded: Uint128,
    requested: Uint128,
    swapped_in: Uint128,
    swapped_out: Uint128,
    swap_fee_rate: Decimal,
) -> StdResult<Decimal> {
    const MAX_RATIO: u128 = 1_000_000;

    let max_ratio = u128_to_dec(MAX_RATIO);
    let one = Decimal::one();

    if swap_fee_rate >= one {
        Err(StdError::generic_err(
            "swap_fee_rate >= one at calc_volume_ratio",
        ))?
    }

    let volume_in = u128_to_dec(bonded) + (one - swap_fee_rate) * u128_to_dec(swapped_in);
    let volume_out = u128_to_dec(requested + swapped_out);

    if !volume_in.is_zero() && !volume_out.is_zero() {
        Ok((volume_out / volume_in).clamp(one / max_ratio, max_ratio))
    } else if !volume_out.is_zero() {
        Ok(max_ratio)
    } else {
        Ok(one / max_ratio)
    }
}

// provider_rewards = provider_power * swap_fee
// swap_fee = swap_fee_rate * amount_in
// provider_power = sum_for_each_asset(allocation * token_weight)
// allocation = asset_bonded / sum_for_each_provider(asset_bonded)
// token_weight = volume_ratio / sum_for_each_token(volume_ratio)
pub fn calc_provider_rewards(
    amount_in: Uint128,
    token_in_price: Decimal,
    token_out_price: Decimal,
    swap_fee_rate: Decimal,
    provider_list: Vec<(Addr, Vec<Asset>)>,
    token_list: Vec<(Addr, Token)>,
) -> StdResult<(Vec<(Addr, Uint128)>, Uint128)> {
    let mut provider_rewards_list: Vec<(Addr, Uint128)> = vec![];

    // distribute rewards to providers
    let swap_fee = swap_fee_rate * u128_to_dec(amount_in);

    // get total values for volume ratio and bonded tokens
    let mut volume_ratio_list: Vec<(Addr, Decimal)> = vec![];
    let mut volume_ratio_sum = Decimal::zero();
    let mut bonded_total: Vec<(Addr, Uint128)> = vec![];

    for (token_addr, token) in token_list {
        let bonded_by_all_providers =
            provider_list
                .iter()
                .fold(Uint128::zero(), |acc, (_, asset_list)| {
                    let asset_default = Asset::new(&token_addr, &Timestamp::default());

                    let asset = asset_list
                        .iter()
                        .find(|x| x.token_addr == token_addr)
                        .unwrap_or(&asset_default);

                    acc + asset.bonded
                });

        let volume_ratio = calc_volume_ratio(
            token.bonded.1,
            token.requested.1,
            token.swapped_in.1,
            token.swapped_out.1,
            swap_fee_rate,
        )?;

        volume_ratio_list.push((token_addr.clone(), volume_ratio));
        volume_ratio_sum += volume_ratio;

        bonded_total.push((token_addr, bonded_by_all_providers));
    }

    // calc rewards for each provider
    for (provider_addr, asset_list) in provider_list {
        let mut provider_power = Decimal::zero();

        for asset in asset_list {
            // calc allocation
            if asset.bonded.is_zero() {
                continue;
            }

            let bonded_default = (asset.token_addr.clone(), Uint128::zero());

            let (_, bonded_total_amount) = bonded_total
                .iter()
                .find(|(addr, _)| addr == &asset.token_addr)
                .unwrap_or(&bonded_default);

            let allocation = if bonded_total_amount.is_zero() {
                Decimal::one()
            } else {
                u128_to_dec(asset.bonded) / u128_to_dec(*bonded_total_amount)
            };

            let volume_ratio_default = (asset.token_addr.clone(), Decimal::zero());

            let (_, volume_ratio_value) = volume_ratio_list
                .iter()
                .find(|(addr, _)| addr == &asset.token_addr)
                .unwrap_or(&volume_ratio_default);

            let token_weight = volume_ratio_value / volume_ratio_sum;
            let asset_power = allocation * token_weight;

            provider_power += asset_power;
        }

        let provider_rewards = (provider_power * swap_fee).to_uint_floor();
        provider_rewards_list.push((provider_addr, provider_rewards));
    }

    let amount_in_clean = u128_to_dec(amount_in) - swap_fee;
    let cost_in = token_in_price * amount_in_clean;
    let amount_out = (cost_in / token_out_price).to_uint_floor();

    Ok((provider_rewards_list, amount_out))
}

#[cfg(test)]
pub mod test {
    use cosmwasm_std::Decimal;

    use super::{
        calc_area, calc_average, calc_provider_rewards, calc_sma, calc_volume_ratio, frame_list,
        interpolate, str_to_dec, u128_to_dec, Addr, Asset, Sample, StdError, Timestamp, Token,
        Uint128,
    };

    use crate::{
        actions::instantiate::SWAP_FEE_RATE,
        tests::helpers::{
            ADDR_ALICE_INJ, ADDR_BOB_INJ, PRICE_ATOM, PRICE_FEED_ID_STR_ATOM,
            PRICE_FEED_ID_STR_LUNA, PRICE_LUNA, SYMBOL_ATOM, SYMBOL_LUNA, TOKEN_ADDR_ATOM,
            TOKEN_ADDR_LUNA,
        },
    };

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
        let requested = Uint128::from(500u128);
        let swapped_in = Uint128::from(1000u128);
        let swapped_out = Uint128::from(1000u128);
        let swap_fee_rate = str_to_dec("0.003");

        let res = calc_volume_ratio(bonded, requested, swapped_in, swapped_out, swap_fee_rate);

        let volume_ratio = str_to_dec("0.751126690035052578");

        assert_eq!(res.unwrap(), volume_ratio);
    }

    #[test]
    fn calc_volume_ratio_fee_is_too_large() {
        let bonded = Uint128::from(1000u128);
        let requested = Uint128::from(500u128);
        let swapped_in = Uint128::from(1000u128);
        let swapped_out = Uint128::from(1000u128);
        let swap_fee_rate = str_to_dec("1");

        let res = calc_volume_ratio(bonded, requested, swapped_in, swapped_out, swap_fee_rate);

        assert_eq!(
            res.unwrap_err(),
            StdError::generic_err("swap_fee_rate >= one at calc_volume_ratio")
        );
    }

    #[test]
    fn calc_volume_ratio_limit_lower() {
        let bonded = Uint128::from(100_000_000u128);
        let requested = Uint128::one();
        let swapped_in = Uint128::one();
        let swapped_out = Uint128::one();
        let swap_fee_rate = str_to_dec("0.003");

        let res = calc_volume_ratio(bonded, requested, swapped_in, swapped_out, swap_fee_rate);

        let volume_ratio = str_to_dec("0.000001");

        assert_eq!(res.unwrap(), volume_ratio);
    }

    #[test]
    fn calc_volume_ratio_limit_upper() {
        let bonded = Uint128::one();
        let requested = Uint128::from(100_000_000u128);
        let swapped_in = Uint128::one();
        let swapped_out = Uint128::one();
        let swap_fee_rate = str_to_dec("0.003");

        let res = calc_volume_ratio(bonded, requested, swapped_in, swapped_out, swap_fee_rate);

        let volume_ratio = str_to_dec("1000000");

        assert_eq!(res.unwrap(), volume_ratio);
    }

    #[test]
    fn calc_provider_rewards_2_providers_2_assets_each() {
        const AMOUNT_IN: u128 = 1_000_000;
        const ALICE_ATOM_ALLOCATION_WEIGHT: &str = "0.2";
        const ALICE_LUNA_ALLOCATION_WEIGHT: &str = "0.5";
        const BONDED_VOLUME_ATOM: u128 = 2_000_000;
        const REQUESTED_VOLUME_ATOM: u128 = 2_000_000;
        const BONDED_VOLUME_LUNA: u128 = 2_000_000;
        const REQUESTED_VOLUME_LUNA: u128 = 2_000_000;

        let amount_in = Uint128::from(AMOUNT_IN);
        let token_in_price = str_to_dec(PRICE_ATOM);
        let token_out_price = str_to_dec(PRICE_LUNA);
        let swap_fee_rate = str_to_dec(SWAP_FEE_RATE);

        let provider_list: Vec<(Addr, Vec<Asset>)> = vec![
            (
                Addr::unchecked(ADDR_ALICE_INJ),
                vec![
                    Asset {
                        token_addr: Addr::unchecked(TOKEN_ADDR_ATOM),
                        bonded: (str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT) * u128_to_dec(AMOUNT_IN))
                            .to_uint_floor(),
                        unbonded: Uint128::from(0u128),
                        requested: Uint128::from(0u128),
                        counter: Timestamp::default(),
                        rewards: Uint128::from(0u128),
                    },
                    Asset {
                        token_addr: Addr::unchecked(TOKEN_ADDR_LUNA),
                        bonded: (str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT) * u128_to_dec(AMOUNT_IN))
                            .to_uint_floor(),
                        unbonded: Uint128::from(0u128),
                        requested: Uint128::from(0u128),
                        counter: Timestamp::default(),
                        rewards: Uint128::from(0u128),
                    },
                ],
            ),
            (
                Addr::unchecked(ADDR_BOB_INJ),
                vec![
                    Asset {
                        token_addr: Addr::unchecked(TOKEN_ADDR_ATOM),
                        bonded: ((Decimal::one() - str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT))
                            * u128_to_dec(AMOUNT_IN))
                        .to_uint_floor(),
                        unbonded: Uint128::from(0u128),
                        requested: Uint128::from(0u128),
                        counter: Timestamp::default(),
                        rewards: Uint128::from(0u128),
                    },
                    Asset {
                        token_addr: Addr::unchecked(TOKEN_ADDR_LUNA),
                        bonded: ((Decimal::one() - str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT))
                            * u128_to_dec(AMOUNT_IN))
                        .to_uint_floor(),
                        unbonded: Uint128::from(0u128),
                        requested: Uint128::from(0u128),
                        counter: Timestamp::default(),
                        rewards: Uint128::from(0u128),
                    },
                ],
            ),
        ];

        let token_list: Vec<(Addr, Token)> = vec![
            (
                Addr::unchecked(TOKEN_ADDR_ATOM),
                Token {
                    symbol: SYMBOL_ATOM.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_ATOM.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_ATOM)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_ATOM)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
            (
                Addr::unchecked(TOKEN_ADDR_LUNA),
                Token {
                    symbol: SYMBOL_LUNA.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_LUNA.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_LUNA)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_LUNA)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
        ];

        let (provider_rewards_list, amount_out) = calc_provider_rewards(
            amount_in,
            token_in_price,
            token_out_price,
            swap_fee_rate,
            provider_list,
            token_list,
        )
        .unwrap();

        let amount_out_right = ((Decimal::one() - str_to_dec(SWAP_FEE_RATE))
            * u128_to_dec(AMOUNT_IN)
            * str_to_dec(PRICE_ATOM)
            / str_to_dec(PRICE_LUNA))
        .to_uint_floor();

        let swap_fee = (str_to_dec(SWAP_FEE_RATE) * u128_to_dec(AMOUNT_IN)).to_uint_ceil();
        let alice_rewards = ((str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT)
            * u128_to_dec(BONDED_VOLUME_ATOM)
            / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA))
            + str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT) * u128_to_dec(BONDED_VOLUME_LUNA)
                / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA)))
            * u128_to_dec(swap_fee))
        .to_uint_floor();
        let bob_rewards = (((Decimal::one() - str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT))
            * u128_to_dec(BONDED_VOLUME_ATOM)
            / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA))
            + (Decimal::one() - str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT))
                * u128_to_dec(BONDED_VOLUME_LUNA)
                / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA)))
            * u128_to_dec(swap_fee))
        .to_uint_floor();

        assert_eq!(amount_out, amount_out_right);
        assert_eq!(
            provider_rewards_list,
            vec![
                (Addr::unchecked(ADDR_ALICE_INJ), alice_rewards),
                (Addr::unchecked(ADDR_BOB_INJ), bob_rewards)
            ]
        );
    }

    #[test]
    fn calc_provider_rewards_2_providers_1_asset_each() {
        const AMOUNT_IN: u128 = 1_000_000;
        const ALICE_ATOM_ALLOCATION_WEIGHT: &str = "1";
        const ALICE_LUNA_ALLOCATION_WEIGHT: &str = "0";
        const BONDED_VOLUME_ATOM: u128 = 2_000_000;
        const REQUESTED_VOLUME_ATOM: u128 = 2_000_000;
        const BONDED_VOLUME_LUNA: u128 = 2_000_000;
        const REQUESTED_VOLUME_LUNA: u128 = 2_000_000;

        let amount_in = Uint128::from(AMOUNT_IN);
        let token_in_price = str_to_dec(PRICE_ATOM);
        let token_out_price = str_to_dec(PRICE_LUNA);
        let swap_fee_rate = str_to_dec(SWAP_FEE_RATE);

        let provider_list: Vec<(Addr, Vec<Asset>)> = vec![
            (
                Addr::unchecked(ADDR_ALICE_INJ),
                vec![Asset {
                    token_addr: Addr::unchecked(TOKEN_ADDR_ATOM),
                    bonded: (str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT) * u128_to_dec(AMOUNT_IN))
                        .to_uint_floor(),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(0u128),
                    counter: Timestamp::default(),
                    rewards: Uint128::from(0u128),
                }],
            ),
            (
                Addr::unchecked(ADDR_BOB_INJ),
                vec![Asset {
                    token_addr: Addr::unchecked(TOKEN_ADDR_LUNA),
                    bonded: ((Decimal::one() - str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT))
                        * u128_to_dec(AMOUNT_IN))
                    .to_uint_floor(),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(0u128),
                    counter: Timestamp::default(),
                    rewards: Uint128::from(0u128),
                }],
            ),
        ];

        let token_list: Vec<(Addr, Token)> = vec![
            (
                Addr::unchecked(TOKEN_ADDR_ATOM),
                Token {
                    symbol: SYMBOL_ATOM.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_ATOM.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_ATOM)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_ATOM)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
            (
                Addr::unchecked(TOKEN_ADDR_LUNA),
                Token {
                    symbol: SYMBOL_LUNA.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_LUNA.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_LUNA)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_LUNA)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
        ];

        let (provider_rewards_list, amount_out) = calc_provider_rewards(
            amount_in,
            token_in_price,
            token_out_price,
            swap_fee_rate,
            provider_list,
            token_list,
        )
        .unwrap();

        let amount_out_right = ((Decimal::one() - str_to_dec(SWAP_FEE_RATE))
            * u128_to_dec(AMOUNT_IN)
            * str_to_dec(PRICE_ATOM)
            / str_to_dec(PRICE_LUNA))
        .to_uint_floor();

        let swap_fee = (str_to_dec(SWAP_FEE_RATE) * u128_to_dec(AMOUNT_IN)).to_uint_ceil();
        let alice_rewards = ((str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT)
            * u128_to_dec(BONDED_VOLUME_ATOM)
            / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA))
            + str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT) * u128_to_dec(BONDED_VOLUME_LUNA)
                / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA)))
            * u128_to_dec(swap_fee))
        .to_uint_floor();
        let bob_rewards = (((Decimal::one() - str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT))
            * u128_to_dec(BONDED_VOLUME_ATOM)
            / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA))
            + (Decimal::one() - str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT))
                * u128_to_dec(BONDED_VOLUME_LUNA)
                / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA)))
            * u128_to_dec(swap_fee))
        .to_uint_floor();

        assert_eq!(amount_out, amount_out_right);
        assert_eq!(
            provider_rewards_list,
            vec![
                (Addr::unchecked(ADDR_ALICE_INJ), alice_rewards),
                (Addr::unchecked(ADDR_BOB_INJ), bob_rewards)
            ]
        );
    }

    #[test]
    fn calc_provider_rewards_1_provider_2_assets() {
        const AMOUNT_IN: u128 = 1_000_000;
        const ALICE_ATOM_ALLOCATION_WEIGHT: &str = "1";
        const ALICE_LUNA_ALLOCATION_WEIGHT: &str = "1";
        const BONDED_VOLUME_ATOM: u128 = 2_000_000;
        const REQUESTED_VOLUME_ATOM: u128 = 2_000_000;
        const BONDED_VOLUME_LUNA: u128 = 2_000_000;
        const REQUESTED_VOLUME_LUNA: u128 = 2_000_000;

        let amount_in = Uint128::from(AMOUNT_IN);
        let token_in_price = str_to_dec(PRICE_ATOM);
        let token_out_price = str_to_dec(PRICE_LUNA);
        let swap_fee_rate = str_to_dec(SWAP_FEE_RATE);

        let provider_list: Vec<(Addr, Vec<Asset>)> = vec![(
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![
                Asset {
                    token_addr: Addr::unchecked(TOKEN_ADDR_ATOM),
                    bonded: (str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT) * u128_to_dec(AMOUNT_IN))
                        .to_uint_floor(),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(0u128),
                    counter: Timestamp::default(),
                    rewards: Uint128::from(0u128),
                },
                Asset {
                    token_addr: Addr::unchecked(TOKEN_ADDR_LUNA),
                    bonded: (str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT) * u128_to_dec(AMOUNT_IN))
                        .to_uint_floor(),
                    unbonded: Uint128::from(0u128),
                    requested: Uint128::from(0u128),
                    counter: Timestamp::default(),
                    rewards: Uint128::from(0u128),
                },
            ],
        )];

        let token_list: Vec<(Addr, Token)> = vec![
            (
                Addr::unchecked(TOKEN_ADDR_ATOM),
                Token {
                    symbol: SYMBOL_ATOM.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_ATOM.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_ATOM)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_ATOM)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
            (
                Addr::unchecked(TOKEN_ADDR_LUNA),
                Token {
                    symbol: SYMBOL_LUNA.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_LUNA.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_LUNA)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_LUNA)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
        ];

        let (provider_rewards_list, amount_out) = calc_provider_rewards(
            amount_in,
            token_in_price,
            token_out_price,
            swap_fee_rate,
            provider_list,
            token_list,
        )
        .unwrap();

        let amount_out_right = ((Decimal::one() - str_to_dec(SWAP_FEE_RATE))
            * u128_to_dec(AMOUNT_IN)
            * str_to_dec(PRICE_ATOM)
            / str_to_dec(PRICE_LUNA))
        .to_uint_floor();

        let swap_fee = (str_to_dec(SWAP_FEE_RATE) * u128_to_dec(AMOUNT_IN)).to_uint_ceil();
        let alice_rewards = ((str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT)
            * u128_to_dec(BONDED_VOLUME_ATOM)
            / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA))
            + str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT) * u128_to_dec(BONDED_VOLUME_LUNA)
                / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA)))
            * u128_to_dec(swap_fee))
        .to_uint_floor();

        assert_eq!(amount_out, amount_out_right);
        assert_eq!(
            provider_rewards_list,
            vec![(Addr::unchecked(ADDR_ALICE_INJ), alice_rewards),]
        );
    }

    #[test]
    fn calc_provider_rewards_1_provider_1_asset() {
        const AMOUNT_IN: u128 = 1_000_000;
        const ALICE_ATOM_ALLOCATION_WEIGHT: &str = "1";
        const ALICE_LUNA_ALLOCATION_WEIGHT: &str = "0";
        const BONDED_VOLUME_ATOM: u128 = 2_000_000;
        const REQUESTED_VOLUME_ATOM: u128 = 2_000_000;
        const BONDED_VOLUME_LUNA: u128 = 2_000_000;
        const REQUESTED_VOLUME_LUNA: u128 = 2_000_000;

        let amount_in = Uint128::from(AMOUNT_IN);
        let token_in_price = str_to_dec(PRICE_ATOM);
        let token_out_price = str_to_dec(PRICE_LUNA);
        let swap_fee_rate = str_to_dec(SWAP_FEE_RATE);

        let provider_list: Vec<(Addr, Vec<Asset>)> = vec![(
            Addr::unchecked(ADDR_ALICE_INJ),
            vec![Asset {
                token_addr: Addr::unchecked(TOKEN_ADDR_ATOM),
                bonded: (str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT) * u128_to_dec(AMOUNT_IN))
                    .to_uint_floor(),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(0u128),
                counter: Timestamp::default(),
                rewards: Uint128::from(0u128),
            }],
        )];

        let token_list: Vec<(Addr, Token)> = vec![
            (
                Addr::unchecked(TOKEN_ADDR_ATOM),
                Token {
                    symbol: SYMBOL_ATOM.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_ATOM.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_ATOM)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_ATOM)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
            (
                Addr::unchecked(TOKEN_ADDR_LUNA),
                Token {
                    symbol: SYMBOL_LUNA.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_LUNA.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_LUNA)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_LUNA)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
        ];

        let (provider_rewards_list, amount_out) = calc_provider_rewards(
            amount_in,
            token_in_price,
            token_out_price,
            swap_fee_rate,
            provider_list,
            token_list,
        )
        .unwrap();

        let amount_out_right = ((Decimal::one() - str_to_dec(SWAP_FEE_RATE))
            * u128_to_dec(AMOUNT_IN)
            * str_to_dec(PRICE_ATOM)
            / str_to_dec(PRICE_LUNA))
        .to_uint_floor();

        let swap_fee = (str_to_dec(SWAP_FEE_RATE) * u128_to_dec(AMOUNT_IN)).to_uint_ceil();
        let alice_rewards = ((str_to_dec(ALICE_ATOM_ALLOCATION_WEIGHT)
            * u128_to_dec(BONDED_VOLUME_ATOM)
            / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA))
            + str_to_dec(ALICE_LUNA_ALLOCATION_WEIGHT) * u128_to_dec(BONDED_VOLUME_LUNA)
                / (u128_to_dec(BONDED_VOLUME_ATOM) + u128_to_dec(BONDED_VOLUME_LUNA)))
            * u128_to_dec(swap_fee))
        .to_uint_floor();

        assert_eq!(amount_out, amount_out_right);
        assert_eq!(
            provider_rewards_list,
            vec![(Addr::unchecked(ADDR_ALICE_INJ), alice_rewards),]
        );
    }

    #[test]
    fn calc_provider_rewards_no_providers() {
        const AMOUNT_IN: u128 = 1_000_000;
        const BONDED_VOLUME_ATOM: u128 = 2_000_000;
        const REQUESTED_VOLUME_ATOM: u128 = 2_000_000;
        const BONDED_VOLUME_LUNA: u128 = 2_000_000;
        const REQUESTED_VOLUME_LUNA: u128 = 2_000_000;

        let amount_in = Uint128::from(AMOUNT_IN);
        let token_in_price = str_to_dec(PRICE_ATOM);
        let token_out_price = str_to_dec(PRICE_LUNA);
        let swap_fee_rate = str_to_dec(SWAP_FEE_RATE);

        let provider_list: Vec<(Addr, Vec<Asset>)> = vec![];

        let token_list: Vec<(Addr, Token)> = vec![
            (
                Addr::unchecked(TOKEN_ADDR_ATOM),
                Token {
                    symbol: SYMBOL_ATOM.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_ATOM.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_ATOM)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_ATOM)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
            (
                Addr::unchecked(TOKEN_ADDR_LUNA),
                Token {
                    symbol: SYMBOL_LUNA.to_string(),
                    price_feed_id_str: PRICE_FEED_ID_STR_LUNA.to_string(),
                    bonded: (vec![], Uint128::from(BONDED_VOLUME_LUNA)),
                    requested: (vec![], Uint128::from(REQUESTED_VOLUME_LUNA)),
                    swapped_in: (vec![], Uint128::from(0u128)),
                    swapped_out: (vec![], Uint128::from(0u128)),
                },
            ),
        ];

        let (provider_rewards_list, amount_out) = calc_provider_rewards(
            amount_in,
            token_in_price,
            token_out_price,
            swap_fee_rate,
            provider_list,
            token_list,
        )
        .unwrap();

        let amount_out_right = ((Decimal::one() - str_to_dec(SWAP_FEE_RATE))
            * u128_to_dec(AMOUNT_IN)
            * str_to_dec(PRICE_ATOM)
            / str_to_dec(PRICE_LUNA))
        .to_uint_floor();

        assert_eq!(amount_out, amount_out_right);
        assert_eq!(provider_rewards_list, vec![]);
    }

    // TODO: add more tests - noisy numbers
}
