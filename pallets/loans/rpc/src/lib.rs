use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_loans_rpc_runtime_api::LoansApi as LoansRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::FixedU128;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

#[rpc]
pub trait LoansApi<BlockHash, AccountId> {
    #[rpc(name = "pallet_loans_get_account_liquidity")]
    fn get_account_liquidity(
        &self,
        account: AccountId,
        at: Option<BlockHash>,
    ) -> Result<(FixedU128, FixedU128)>;
}

/// A struct that implements the [`LoansApi`].
pub struct Loans<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, B> Loans<C, B> {
    /// Create new `Loans` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

pub enum Error {
    RuntimeError,
    AccountLiquidityError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::AccountLiquidityError => 2,
        }
    }
}

impl<C, Block, AccountId> LoansApi<<Block as BlockT>::Hash, AccountId> for Loans<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: LoansRuntimeApi<Block, AccountId>,
    AccountId: Codec,
{
    fn get_account_liquidity(
        &self,
        account: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<(FixedU128, FixedU128)> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or(
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash,
        ));
        api.get_account_liquidity(&at, account)
            .map_err(runtime_error_into_rpc_error)?
            .map_err(account_liquidity_error_into_rpc_error)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::RuntimeError.into()),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}

/// Converts an account liquidity error into an RPC error.
fn account_liquidity_error_into_rpc_error(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(Error::AccountLiquidityError.into()),
        message: "Not able to get account liquidity".into(),
        data: Some(format!("{:?}", err).into()),
    }
}
