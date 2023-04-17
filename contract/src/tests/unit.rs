use cosmwasm_std::{Addr, Timestamp, Uint128};

use cw20::Cw20Coin;

use crate::{
    state::Asset,
    tests::helpers::{
        Project, ADDR_ADMIN_INJ, ADDR_ALICE_INJ, ADDR_BOB_INJ, PRICE_FEED_ID_STR_ATOM,
        PRICE_FEED_ID_STR_LUNA, SYMBOL_ATOM, SYMBOL_LUNA, UNBONDING_PERIOD,
    },
};

fn default_init() -> (Project, Addr, Cw20Coin) {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mut prj = Project::new();

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    (prj, token, mint_amount)
}

#[test]
fn create_cw20() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mut prj = Project::new();

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    let balance = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance, mint_amount.amount);
}

#[test]
fn deposit() {
    let (mut prj, token, mint_amount) = default_init();

    let balance_contract = prj.get_cw20_balance(token.clone(), prj.address.clone());
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, mint_amount.amount);
    assert_eq!(balance_alice, Uint128::zero());
}

#[test]
#[should_panic(expected = "Token is not included in token list!")]
fn deposit_unsupported_token() {
    let (mut prj, _token, mint_amount) = default_init();

    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone()]);

    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();
}

#[test]
#[should_panic(expected = "There are not enough funds to withdraw!")]
fn withdraw_without_unbond() {
    let (mut prj, token, mint_amount) = default_init();

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
}

#[test]
#[should_panic(expected = "There are not enough funds to withdraw!")]
fn withdraw_with_unbond_but_too_early() {
    let (mut prj, token, mint_amount) = default_init();

    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
}

#[test]
fn withdraw() {
    let (mut prj, token, mint_amount) = default_init();

    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    prj.wait(UNBONDING_PERIOD as u64);

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    let balance_contract = prj.get_cw20_balance(token.clone(), prj.address.clone());
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, Uint128::zero());
    assert_eq!(balance_alice, mint_amount.amount);
}

#[test]
fn deposit_unbond_withdraw_loop() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(9u128),
    };

    let mut prj = Project::new();

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    // deposit 9 ATOM
    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    // unbond 3 ATOM
    prj.unbond(ADDR_ALICE_INJ, &token, Uint128::from(3u128))
        .unwrap();

    // wait UNBONDING_PERIOD
    prj.wait(UNBONDING_PERIOD as u64);

    // withdraw 1 ATOM
    prj.withdraw(ADDR_ALICE_INJ, &token, Uint128::from(1u128))
        .unwrap();

    // unbond 1 ATOM
    prj.unbond(ADDR_ALICE_INJ, &token, Uint128::from(1u128))
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![Asset {
            token_addr: Addr::unchecked("contract1"),
            bonded: Uint128::from(5u128),
            unbonded: Uint128::from(2u128),
            requested: Uint128::from(1u128),
            counter: Timestamp::from_nanos(1571804619879305533u64),
            rewards: Uint128::from(0u128),
        }]
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    // unbond 5 ATOM
    prj.unbond(ADDR_ALICE_INJ, &token, Uint128::from(5u128))
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![Asset {
            token_addr: Addr::unchecked("contract1"),
            bonded: Uint128::from(0u128),
            unbonded: Uint128::from(2u128),
            requested: Uint128::from(6u128),
            counter: Timestamp::from_nanos(1571806419879305533u64),
            rewards: Uint128::from(0u128),
        }]
    );

    // deposit 9 LUNA
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(2u128),
                requested: Uint128::from(6u128),
                counter: Timestamp::from_nanos(1571806419879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(9u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(0u128),
                counter: Timestamp::from_nanos(1571802819879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    // unbond 3 LUNA
    prj.unbond(ADDR_ALICE_INJ, &token2, Uint128::from(3u128))
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(2u128),
                requested: Uint128::from(6u128),
                counter: Timestamp::from_nanos(1571806419879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(6u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(3u128),
                counter: Timestamp::from_nanos(1571808219879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    // unbond 3 LUNA
    prj.unbond(ADDR_ALICE_INJ, &token2, Uint128::from(3u128))
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(8u128),
                requested: Uint128::from(0u128),
                counter: Timestamp::from_nanos(1571806419879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(3u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(6u128),
                counter: Timestamp::from_nanos(1571810019879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );

    // withdraw 8 ATOM
    prj.withdraw(ADDR_ALICE_INJ, &token, Uint128::from(8u128))
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![Asset {
            token_addr: Addr::unchecked("contract2"),
            bonded: Uint128::from(3u128),
            unbonded: Uint128::from(0u128),
            requested: Uint128::from(6u128),
            counter: Timestamp::from_nanos(1571810019879305533u64),
            rewards: Uint128::from(0u128),
        },]
    );

    // unbond 3 LUNA
    prj.unbond(ADDR_ALICE_INJ, &token2, Uint128::from(3u128))
        .unwrap();

    // wait UNBONDING_PERIOD
    prj.wait((UNBONDING_PERIOD) as u64);

    // withdraw 9 LUNA
    prj.withdraw(ADDR_ALICE_INJ, &token2, Uint128::from(9u128))
        .unwrap();

    assert_eq!(prj.query_provider(ADDR_ALICE_INJ).unwrap(), vec![]);
}

#[test]
fn deposit_2_providers() {
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mint_amount2 = Cw20Coin {
        address: ADDR_BOB_INJ.to_string(),
        amount: Uint128::from(50u128),
    };

    let mut prj = Project::new();

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone(), mint_amount2.clone()]);
    let token2 = prj.create_cw20(SYMBOL_LUNA, vec![mint_amount.clone(), mint_amount2.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();
    prj.update_token(ADDR_ADMIN_INJ, &token2, SYMBOL_LUNA, PRICE_FEED_ID_STR_LUNA)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.deposit(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    prj.unbond(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.unbond(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(5u128),
                counter: Timestamp::from_nanos(1571801019879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(5u128),
                counter: Timestamp::from_nanos(1571801019879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    prj.deposit(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.deposit(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    prj.unbond(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.unbond(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    assert_eq!(
        prj.query_provider(ADDR_ALICE_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(5u128),
                counter: Timestamp::from_nanos(1571801019879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(5u128),
                counter: Timestamp::from_nanos(1571801019879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );
    assert_eq!(
        prj.query_provider(ADDR_BOB_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(50u128),
                counter: Timestamp::from_nanos(1571802819879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(50u128),
                counter: Timestamp::from_nanos(1571802819879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    prj.withdraw(ADDR_ALICE_INJ, &token2, mint_amount.amount)
        .unwrap();

    assert_eq!(prj.query_provider(ADDR_ALICE_INJ).unwrap(), vec![]);
    assert_eq!(
        prj.query_provider(ADDR_BOB_INJ).unwrap(),
        vec![
            Asset {
                token_addr: Addr::unchecked("contract1"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(50u128),
                counter: Timestamp::from_nanos(1571802819879305533u64),
                rewards: Uint128::from(0u128),
            },
            Asset {
                token_addr: Addr::unchecked("contract2"),
                bonded: Uint128::from(0u128),
                unbonded: Uint128::from(0u128),
                requested: Uint128::from(50u128),
                counter: Timestamp::from_nanos(1571802819879305533u64),
                rewards: Uint128::from(0u128),
            },
        ]
    );

    // wait UNBONDING_PERIOD / 2
    prj.wait((UNBONDING_PERIOD / 2) as u64);

    prj.withdraw(ADDR_BOB_INJ, &token, mint_amount2.amount)
        .unwrap();
    prj.withdraw(ADDR_BOB_INJ, &token2, mint_amount2.amount)
        .unwrap();

    assert_eq!(prj.query_provider(ADDR_ALICE_INJ).unwrap(), vec![]);
    assert_eq!(prj.query_provider(ADDR_BOB_INJ).unwrap(), vec![]);
}
