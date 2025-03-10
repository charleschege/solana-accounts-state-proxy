use crate::{Account, AccountInfo, Context, Encoding};
use jsonrpsee::core::RpcResult;
use serde_json::Value as SerdeJsonValue;
use tokio_postgres::Row;

#[cfg(debug_assertions)]
use tokio::time::Instant;

/// Enables easier serialization from a postgres `Row` from the `getAccountInfo` query
#[derive(Debug)]
pub struct GetAccountInfoRow {
    pub(crate) context: Context,
    pub(crate) value: Account,
}

impl From<Row> for GetAccountInfoRow {
    fn from(row: Row) -> Self {
        let slot: i64 = row.get(0);
        let slot = slot as u64;
        let data: Vec<u8> = row.get(1);
        let executable: bool = row.get(2);
        let owner: String = row.get(3);
        let lamports: i64 = row.get(4);
        let rent_epoch: i64 = row.get(5);

        GetAccountInfoRow {
            context: Context {
                slot,
                api_version: Option::None,
            },
            value: Account {
                data,
                executable,
                owner,
                lamports,
                rent_epoch,
            },
        }
    }
}

/// Enables easier serialization from a postgres `Row` from the `getAccountInfo` query
#[derive(Debug)]
pub struct GetProgramAccountsRow;

impl GetProgramAccountsRow {
    /// Convert a postgres Row into [AccountInfo] then to JSON format in one method.
    pub fn from_row(rows: Vec<Row>, encoding: Encoding) -> RpcResult<Vec<SerdeJsonValue>> {
        tracing::debug!("NUMBER OF ROWS TO PARSE: {:?}", &rows.len());
        tracing::debug!("PARSING ROWS AND CONVERTING TO JSON");

        #[cfg(debug_assertions)]
        let timer = Instant::now();

        // FIXME remove re-allocations in each iter

        use rayon::prelude::*;
        let account_info_list = rows
            .par_iter()
            .map(|row| {
                let pubkey: String = row.get(0);
                let lamports: i64 = row.get(1);
                let owner: String = row.get(2);
                let executable: bool = row.get(3);
                let rent_epoch: i64 = row.get(4);
                let data: Vec<u8> = row.get(5);

                let account = Account {
                    data,
                    executable,
                    owner,
                    lamports,
                    rent_epoch,
                };

                let account_info = AccountInfo { pubkey, account };
                let to_json = account_info.as_json_value(encoding);

                to_json
            })
            .collect::<RpcResult<Vec<SerdeJsonValue>>>();

        #[cfg(debug_assertions)]
        let outcome = Instant::now().duration_since(timer);

        #[cfg(debug_assertions)]
        tracing::debug!(
            "FINISHED PARSING ROWS AND CONVERTING TO JSON IN {}s",
            outcome.as_secs()
        );

        #[cfg(debug_assertions)]
        let mut mb = 0usize;

        let account_info_list = account_info_list?;

        #[cfg(debug_assertions)]
        for chunk in &account_info_list {
            mb += chunk.to_string().as_bytes().len();
        }

        #[cfg(debug_assertions)]
        let to_mb = mb as f32 / 1024.0 / 1024.0;

        #[cfg(debug_assertions)]
        tracing::debug!("TOTAL LENGTH OF DATA IN MB - {}", to_mb);

        Ok(account_info_list)
    }
}
