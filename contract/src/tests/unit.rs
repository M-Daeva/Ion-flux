use cosmwasm_std::Uint128;

use cw20::Cw20Coin;

use crate::tests::helpers::{Project, ADDR_ALICE_INJ, SYMBOL_ATOM};

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
    prj.deposit(ADDR_ALICE_INJ, token.clone(), mint_amount.amount)
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
        address: contract_address.to_string(),
        amount: Uint128::from(5u128),
    };
    let token = prj.create_cw20(SYMBOL_ATOM, vec![mint_amount.clone()]);
    prj.withdraw(ADDR_ALICE_INJ, token.as_str(), mint_amount.amount)
        .unwrap();
    let balance_contract = prj.get_cw20_balance(token.clone(), contract_address);
    let balance_alice = prj.get_cw20_balance(token, ADDR_ALICE_INJ);

    assert_eq!(balance_contract, Uint128::zero());
    assert_eq!(balance_alice, mint_amount.amount);
}
