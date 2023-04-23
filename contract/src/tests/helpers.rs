use cosmwasm_std::{to_binary, Addr, Decimal, Empty, StdResult, Uint128};

use cw20::Cw20Coin;

use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};

use crate::{
    contract::{execute, instantiate, query},
    messages::{execute::ExecuteMsg, query::QueryMsg, receive::ReceiveMsg, response::Balance},
    state::{Asset, Config, Token, CHAIN_ID_MOCKED},
};

pub const ADDR_ADMIN_INJ: &str = "inj1amp7dv5fvjyx95ea4grld6jmu9v207awtefwce";
pub const ADDR_ALICE_INJ: &str = "inj1hag3kx8f9ypnssw7aqnq9e82t2zgt0g0ac2rru";
pub const ADDR_BOB_INJ: &str = "inj1prmtvxpvdcmp3dtn6qn4hyq9gytj5ry4u28nqz";

pub const SYMBOL_ATOM: &str = "ATOM";
pub const SYMBOL_LUNA: &str = "LUNA";
// pub const SYMBOL_USDC: &str = "USDC";
// pub const SYMBOL_OSMO: &str = "OSMO";

pub const PRICE_ATOM: &str = "10";
pub const PRICE_LUNA: &str = "2";

pub const TOKEN_ADDR_ATOM: &str = "token_addr_atom";
pub const TOKEN_ADDR_LUNA: &str = "token_addr_luna";

pub const PRICE_FEED_ID_STR_ATOM: &str =
    "0x61226d39beea19d334f17c2febce27e12646d84675924ebb02b9cdaea68727e3";
pub const PRICE_FEED_ID_STR_LUNA: &str =
    "0x677dbbf4f68b5cb996a40dfae338b87d5efb2e12a9b2686d1ca16d69b3d7f204";
// pub const PRICE_FEED_ID_STR_USDC: &str =
//     "0x41f3625971ca2ed2263e78573fe5ce23e13d2558ed3f2e47ab0f84fb9e7ae722";
// pub const PRICE_FEED_ID_STR_OSMO: &str =
//     "0xd9437c194a4b00ba9d7652cd9af3905e73ee15a2ca4152ac1f8d430cc322b857";

pub const UNBONDING_PERIOD: u128 = 60 * 60 * 1_000_000_000;
pub const SWAP_FEE_RATE: &str = "0.003";

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
        // set specific chain_id to prevent execution of mocked actions on real networks
        app.update_block(|block| block.chain_id = String::from(CHAIN_ID_MOCKED));

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
    pub fn wait(&mut self, delay_ns: u64) {
        self.app.update_block(|block| {
            block.time = block.time.plus_nanos(delay_ns);
            block.height += delay_ns / 5_000_000_000;
        });
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
        price_age: Option<Uint128>,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::UpdateConfig {
                    admin,
                    swap_fee_rate,
                    window,
                    unbonding_period,
                    price_age,
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
    ) -> StdResult<AppResponse> {
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
    ) -> StdResult<AppResponse> {
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
    ) -> StdResult<AppResponse> {
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
    pub fn claim(&mut self, sender: &str) -> StdResult<AppResponse> {
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
    pub fn swap_and_claim_mocked(
        &mut self,
        sender: &str,
        token_out_addr: &Addr,
    ) -> StdResult<AppResponse> {
        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                self.address.clone(),
                &ExecuteMsg::SwapAndClaimMocked {
                    token_out_addr: token_out_addr.to_string(),
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
    ) -> StdResult<AppResponse> {
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
    pub fn swap_mocked(
        &mut self,
        sender: &str,
        amount_in: Uint128,
        token_in_addr: &Addr,
        token_out_addr: &Addr,
    ) -> StdResult<AppResponse> {
        let msg = cw20::Cw20ExecuteMsg::Send {
            contract: self.address.to_string(),
            amount: amount_in,
            msg: to_binary(&ReceiveMsg::SwapMocked {
                token_out_addr: token_out_addr.to_string(),
            })?,
        };

        self.app
            .execute_contract(
                Addr::unchecked(sender.to_string()),
                token_in_addr.to_owned(),
                &msg,
                &[],
            )
            .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn query_config(&self) -> StdResult<Config> {
        self.app
            .wrap()
            .query_wasm_smart(self.address.clone(), &QueryMsg::QueryConfig {})
    }

    #[track_caller]
    pub fn query_aprs(&self, address_list: Vec<&str>) -> StdResult<Vec<(Addr, Decimal)>> {
        self.app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::QueryAprs {
                address_list: address_list.iter().map(|x| x.to_string()).collect(),
            },
        )
    }

    #[track_caller]
    pub fn query_providers(&self, address_list: Vec<&str>) -> StdResult<Vec<(Addr, Vec<Asset>)>> {
        self.app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::QueryProviders {
                address_list: address_list.iter().map(|x| x.to_string()).collect(),
            },
        )
    }

    #[track_caller]
    pub fn query_tokens(&self, address_list: Vec<&str>) -> StdResult<Vec<(Addr, Token)>> {
        self.app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::QueryTokens {
                address_list: address_list.iter().map(|x| x.to_string()).collect(),
            },
        )
    }

    #[track_caller]
    pub fn query_balances(&self, address_list: Vec<&str>) -> StdResult<Vec<Balance>> {
        self.app.wrap().query_wasm_smart(
            self.address.clone(),
            &QueryMsg::QueryBalances {
                address_list: address_list.iter().map(|x| x.to_string()).collect(),
            },
        )
    }
}
