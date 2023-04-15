use cosmwasm_std::StdError;
use thiserror::Error;

pub fn from_std_err(std_error: StdError) -> ContractError {
    ContractError::CustomError {
        val: std_error.to_string(),
    }
}

pub fn to_std_err(contract_error: ContractError) -> StdError {
    StdError::generic_err(contract_error.to_string())
}

/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },

    #[error("Funds are not found!")]
    FundsAreNotFound {},

    #[error("There are not enough funds to withdraw!")]
    WithdrawAmountIsExceeded {},

    #[error("Provider is not found!")]
    ProviderIsNotFound {},

    #[error("Token is not included in token list!")]
    TokenIsNotFound {},

    #[error("Non equal vectors")]
    NonEqualVectors {},

    #[error("Empty vector")]
    EmptyVector {},

    #[error("Sender does not have access permissions!")]
    Unauthorized {},
}
