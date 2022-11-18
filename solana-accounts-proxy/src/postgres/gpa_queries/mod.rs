use crate::{
    Commitment, Context, DataSlice, Encoding, Filter, GetAccountInfoRow, ProxyError, ProxyResult,
};

mod with_confirmed;
pub use with_confirmed::*;

mod with_processed;
pub use with_processed::*;

mod with_finalized;
pub use with_finalized::*;

/// Helper struct for `getProgramAccounts`
#[derive(Debug)]
pub struct GetProgramAccounts<'q> {
    base58_public_key: &'q str,
    commitment: &'q str,
    min_context_slot: Option<u64>,
    data_slice: Option<DataSlice>,
    filters: Option<Vec<Filter>>,
}

impl<'q> GetProgramAccounts<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetProgramAccounts {
            base58_public_key: "",
            commitment: "",
            min_context_slot: Option::default(),
            data_slice: Option::default(),
            filters: Option::default(),
        }
    }

    /// Add a base58 public key
    pub fn add_public_key(mut self, base58_public_key: &'q str) -> Self {
        self.base58_public_key = base58_public_key;

        self
    }

    /// Add the commitment level
    pub fn add_commitment(mut self, commitment: &'q str) -> Self {
        self.commitment = commitment;

        self
    }

    /// Add the minimum context slot
    pub fn add_min_context_slot(mut self, min_context_slot: Option<u64>) -> Self {
        self.min_context_slot = min_context_slot;

        self
    }

    /// Add the data slice
    pub fn add_data_slice(mut self, data_slice: Option<DataSlice>) -> Self {
        self.data_slice = data_slice;

        self
    }

    /// Add the filters for the query
    pub fn add_filters(mut self, filters: Option<Vec<Filter>>) -> Self {
        self.filters = filters;

        self
    }

    /// Executor for the queries
    pub async fn load_data(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        dbg!(&self);

        // Check if only basic queries are supported
        if self.filters.is_none() && self.data_slice.is_none() && self.min_context_slot.is_none() {
            self.basic_with_commitment().await
        } else {
            panic!()
        }
    }

    /// `gPA` accounts with commitment level and an `owner`
    pub async fn basic_with_commitment(&self) -> ProxyResult<Vec<tokio_postgres::Row>> {
        let commitment: Commitment = self.commitment.into();
        let owner = self.base58_public_key;

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        if commitment == Commitment::Processed {
            let rows = pg_client.query("
            SELECT DISTINCT ON(account_write.pubkey) account_write.pubkey FROM account_write 
            WHERE (rooted = TRUE OR slot = (SELECT MAX(slot) FROM slot WHERE slot.status = 'Confirmed' OR slot.status='Processed'))
            AND owner = $1::TEXT
            ORDER BY account_write.pubkey, account_write.slot DESC;
            ", &[&owner]).await?;

            Ok(rows)
        } else if commitment == Commitment::Confirmed {
            let rows = pg_client.query("
            SELECT DISTINCT on(account_write.pubkey) account_write.pubkey FROM account_write 
            WHERE (rooted = TRUE OR slot = (SELECT MAX(slot) FROM slot WHERE slot.status = 'Confirmed') )
            AND owner = $1::TEXT
            ORDER BY account_write.pubkey, account_write.slot DESC;
            ", &[&owner]).await?;

            Ok(rows)
        } else {
            let rows = pg_client.query(
                "
                SELECT DISTINCT on(account_write.pubkey)
                    account_write.pubkey, account_write.owner, account_write.lamports, account_write.executable, account_write.rent_epoch, account_write.data
                FROM account_write
                WHERE
                    rooted = true
                AND owner = $1::TEXT
                ORDER BY account_write.pubkey, account_write.slot DESC;
                ",
                &[&owner]
            ).await?;

            Ok(rows)
        }
    }
}

impl<'q> Default for GetProgramAccounts<'q> {
    fn default() -> Self {
        GetProgramAccounts::new()
    }
}
