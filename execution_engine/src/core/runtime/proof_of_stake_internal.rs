use casper_types::{
    account::AccountHash,
    proof_of_stake::{MintProvider, ProofOfStake, RuntimeProvider},
    system_contract_errors::pos::Error,
    BlockTime, Key, Phase, TransferredTo, URef, U512,
};

use crate::{
    core::{execution, runtime::Runtime},
    shared::stored_value::StoredValue,
    storage::global_state::StateReader,
};

impl From<execution::Error> for Option<Error> {
    fn from(exec_error: execution::Error) -> Self {
        match exec_error {
            // This is used to propagate [`execution::Error::GasLimit`] to make sure
            // [`ProofOfStake`] contract running natively supports propagating gas limit
            // errors without a panic.
            execution::Error::GasLimit => Some(Error::GasLimit),
            // There are possibly other exec errors happening but such translation would be lossy.
            _ => None,
        }
    }
}

// TODO: Update MintProvider to better handle errors
impl<'a, R> MintProvider for Runtime<'a, R>
where
    R: StateReader<Key, StoredValue>,
    R::Error: Into<execution::Error>,
{
    fn transfer_purse_to_account(
        &mut self,
        source: URef,
        target: AccountHash,
        amount: U512,
    ) -> Result<TransferredTo, Error> {
        match self.transfer_from_purse_to_account(source, target, amount, None) {
            Ok(Ok(transferred_to)) => Ok(transferred_to),
            Ok(Err(_mint_error)) => Err(Error::Transfer),
            Err(exec_error) => Err(<Option<Error>>::from(exec_error).unwrap_or(Error::Transfer)),
        }
    }

    fn transfer_purse_to_purse(
        &mut self,
        source: URef,
        target: URef,
        amount: U512,
    ) -> Result<(), Error> {
        let mint_contract_key = self.get_mint_contract();
        match self.mint_transfer(mint_contract_key, None, source, target, amount, None) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_mint_error)) => Err(Error::Transfer),
            Err(exec_error) => Err(<Option<Error>>::from(exec_error).unwrap_or(Error::Transfer)),
        }
    }

    fn balance(&mut self, purse: URef) -> Result<Option<U512>, Error> {
        self.get_balance(purse)
            .map_err(|exec_error| <Option<Error>>::from(exec_error).unwrap_or(Error::GetBalance))
    }
}

// TODO: Update RuntimeProvider to better handle errors
impl<'a, R> RuntimeProvider for Runtime<'a, R>
where
    R: StateReader<Key, StoredValue>,
    R::Error: Into<execution::Error>,
{
    fn get_key(&self, name: &str) -> Result<Option<Key>, Error> {
        Ok(self.context.named_keys_get(name).cloned())
    }

    fn put_key(&mut self, name: &str, key: Key) -> Result<(), Error> {
        self.context
            .put_key(name.to_string(), key)
            .map_err(|exec_error| <Option<Error>>::from(exec_error).unwrap_or(Error::PutKey))
    }

    fn remove_key(&mut self, name: &str) -> Result<(), Error> {
        self.context
            .remove_key(name)
            .map_err(|exec_error| <Option<Error>>::from(exec_error).unwrap_or(Error::RemoveKey))
    }

    fn get_phase(&self) -> Result<Phase, Error> {
        Ok(self.context.phase())
    }

    fn get_block_time(&self) -> Result<BlockTime, Error> {
        Ok(self.context.get_blocktime())
    }

    fn get_caller(&self) -> Result<AccountHash, Error> {
        Ok(self.context.get_caller())
    }
}

impl<'a, R> ProofOfStake for Runtime<'a, R>
where
    R: StateReader<Key, StoredValue>,
    R::Error: Into<execution::Error>,
{
}
