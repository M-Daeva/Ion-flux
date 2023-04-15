use cosmwasm_std::{to_binary, Addr, Decimal, Empty, StdError, Uint128};

use cw20::Cw20Coin;

use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use pyth_sdk_cw::PriceFeedResponse;

use crate::{
    contract::{execute, instantiate, query},
    messages::{execute::ExecuteMsg, query::QueryMsg, receive::ReceiveMsg, response::Balance},
    state::{Asset, Token},
};

pub const ADDR_ADMIN_INJ: &str = "inj1amp7dv5fvjyx95ea4grld6jmu9v207awtefwce";
pub const ADDR_ALICE_INJ: &str = "inj1prmtvxpvdcmp3dtn6qn4hyq9gytj5ry4u28nqz";

pub const SYMBOL_ATOM: &str = "ATOM";

pub const PRICE_FEED_ID_STR_ATOM: &str =
    "0x61226d39beea19d334f17c2febce27e12646d84675924ebb02b9cdaea68727e3";

// pub const TEST_CONTRACT_ADDR: &str = "inj14hj2tavq8fpesdwxxcu44rty3hh90vhujaxlnz";

// pub const PRICE_FEED_ID_INJ_STR: &str =
//     "0x2d9315a88f3019f8efa88dfe9c0f0843712da0bac814461e27733f6b83eb51b3";

pub struct Project {
    pub address: Addr,
    app: App,
}

impl Project {
    pub fn new() -> Self {
        let mut app = Self::create_app();
        let id = Self::store_code(&mut app);
        let address = Self::instantiate(&mut app, id);

        Self { address, app }
    }

    #[track_caller]
    fn create_app() -> App {
        App::default()
    }

    fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    fn instantiate(app: &mut App, id: u64) -> Addr {
        app.instantiate_contract(
            id,
            Addr::unchecked(ADDR_ADMIN_INJ),
            &Empty {},
            &[],
            "Project",
            Some(ADDR_ADMIN_INJ.to_string()),
        )
        .unwrap()
    }

    #[track_caller]
    pub fn create_cw20(&mut self, symbol: &str, initial_balances: Vec<Cw20Coin>) -> Addr {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );

        let id = self.app.store_code(Box::new(contract));

        let msg = cw20_base::msg::InstantiateMsg {
            name: format!("Test CW20 token '{}'", symbol),
            symbol: symbol.to_string(),
            decimals: 6,
            initial_balances,
            mint: None,
            marketing: None,
        };

        self.app
            .instantiate_contract(
                id,
                Addr::unchecked(ADDR_ADMIN_INJ),
                &msg.clone(),
                &[],
                msg.name,
                Some(ADDR_ADMIN_INJ.to_string()),
            )
            .unwrap()
    }

    #[track_caller]
    pub fn get_cw20_balance<T: Into<String>, U: Into<String>>(
        &mut self,
        contract_addr: T,
        address: U,
    ) -> Uint128 {
        let msg = cw20::Cw20QueryMsg::Balance {
            address: address.into(),
        };

        let result: cw20::BalanceResponse = self
            .app
            .wrap()
            .query_wasm_smart(contract_addr, &msg)
            .unwrap();

        result.balance
    }

    #[track_caller]
    pub fn update_config(
        &mut self,
        sender: &str,
        admin: Option<String>,
        swap_fee_rate: Option<Decimal>,
        window: Option<Uint128>,
        unbonding_period: Option<Uint128>,
    ) -> Result<AppResponse, StdError> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::UpdateConfig {
                    admin,
                    swap_fee_rate,
                    window,
                    unbonding_period,
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn update_token(
        &mut self,
        sender: &str,
        token_addr: &Addr,
        symbol: &str,
        price_feed_id_str: &str,
    ) -> Result<AppResponse, StdError> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::UpdateToken {
                    token_addr: token_addr.to_string(),
                    symbol: symbol.to_string(),
                    price_feed_id_str: price_feed_id_str.to_string(),
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn unbond(
        &mut self,
        sender: &str,
        token_addr: &Addr,
        amount: Uint128,
    ) -> Result<AppResponse, StdError> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::Unbond {
                    token_addr: token_addr.to_string(),
                    amount,
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn withdraw(
        &mut self,
        sender: &str,
        token_addr: &Addr,
        amount: Uint128,
    ) -> Result<AppResponse, StdError> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::Withdraw {
                    token_addr: token_addr.to_string(),
                    amount,
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn claim(&mut self, sender: &str) -> Result<AppResponse, StdError> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::Claim {},
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn swap_and_claim(
        &mut self,
        sender: &str,
        token_addr: &Addr,
    ) -> Result<AppResponse, StdError> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::SwapAndClaim {
                    token_addr: token_addr.to_string(),
                },
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn deposit(
        &mut self,
        sender: &str,
        token_addr: &Addr,
        amount: Uint128,
    ) -> Result<AppResponse, StdError> {
        let msg = cw20::Cw20ExecuteMsg::Send {
            contract: self.address.to_string(),
            amount,
            msg: to_binary(&ReceiveMsg::Deposit {})?,
        };

        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                token_addr.to_owned(),
                &msg,
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn swap(
        &mut self,
        sender: &str,
        token_addr: &Addr,
        amount: Uint128,
        token_addr_out: &Addr,
    ) -> Result<AppResponse, StdError> {
        let msg = cw20::Cw20ExecuteMsg::Send {
            contract: self.address.to_string(),
            amount,
            msg: to_binary(&ReceiveMsg::Swap {
                token_addr_out: token_addr_out.to_string(),
            })?,
        };

        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                token_addr.to_owned(),
                &msg,
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn query_provider(&self, address: &str) -> Result<Vec<Asset>, StdError> {
        self.app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::QueryProvider {
                address: address.to_string(),
            },
        )
    }

    #[track_caller]
    pub fn query_tokens(&self) -> Result<Vec<Token>, StdError> {
        self.app
            .wrap()
            .query_wasm_smart(self.address.clone(), &QueryMsg::QueryTokens {})
    }

    #[track_caller]
    pub fn query_balances(&self) -> Result<Vec<Balance>, StdError> {
        self.app
            .wrap()
            .query_wasm_smart(self.address.clone(), &QueryMsg::QueryBalances {})
    }

    // pyth test example
    // https://github.com/pyth-network/pyth-crosschain/blob/main/target_chains/cosmwasm/examples/cw-contract/src/contract.rs
    #[track_caller]
    pub fn query_price(&self, price_feed_id_str: &str) -> Result<PriceFeedResponse, StdError> {
        self.app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::QueryPrice {
                price_feed_id_str: price_feed_id_str.to_string(),
            },
        )
    }

    // pub fn get_attrs(res: &AppResponse) -> Vec<Attribute> {
    //     let mut attrs: Vec<Attribute> = vec![];

    //     for item in &res.events {
    //         for attr in &item.attributes {
    //             attrs.push(attr.to_owned())
    //         }
    //     }

    //     attrs
    // }

    // pub fn get_attr(res: &AppResponse, key: &str) -> String {
    //     let attrs = Self::get_attrs(res);
    //     let attr = attrs.iter().find(|x| x.key == *key).unwrap();

    //     attr.to_owned().value
    // }
}
