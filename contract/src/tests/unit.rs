use cosmwasm_std::Uint128;

use cw20::Cw20Coin;

use crate::tests::helpers::{
    Project, ADDR_ADMIN_INJ, ADDR_ALICE_INJ, PRICE_FEED_ID_STR_ATOM, SYMBOL_ATOM,
};

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
    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let mut prj = Project::new();
    let contract_address = prj.address.clone();

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();

    // let res = prj.query_provider(ADDR_ALICE_INJ).unwrap();
    // println!("provider {:#?}", res);

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    let balance_contract = prj.get_cw20_balance(token.clone(), contract_address);
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, mint_amount.amount);
    assert_eq!(balance_alice, Uint128::zero());
}

#[test]
fn withdraw() {
    let mut prj = Project::new();
    let contract_address = prj.address.clone();

    let mint_amount = Cw20Coin {
        address: ADDR_ALICE_INJ.to_string(),
        amount: Uint128::from(5u128),
    };

    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);

    prj.update_token(ADDR_ADMIN_INJ, &token, SYMBOL_ATOM, PRICE_FEED_ID_STR_ATOM)
        .unwrap();

    prj.deposit(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();

    let res = prj.query_provider(ADDR_ALICE_INJ).unwrap();
    println!("provider {:#?}", res);

    prj.withdraw(ADDR_ALICE_INJ, &token, mint_amount.amount)
        .unwrap();
    let balance_contract = prj.get_cw20_balance(token.clone(), contract_address);
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, Uint128::zero());
    assert_eq!(balance_alice, mint_amount.amount);
}
