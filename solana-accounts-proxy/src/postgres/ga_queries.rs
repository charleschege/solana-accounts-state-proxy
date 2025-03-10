use crate::{Commitment, Context, GetAccountInfoRow, ProxyResult};

/// Helper struct to create the query for `getAccountInfo` using the builder pattern
pub struct GetAccountInfoQuery<'q> {
    base58_public_key: &'q str,
    commitment: &'q str,
    min_context_slot: Option<u64>,
}

impl<'q> GetAccountInfoQuery<'q> {
    /// Instantiate the struct with defaults
    pub fn new() -> Self {
        GetAccountInfoQuery {
            base58_public_key: "",
            commitment: "",
            min_context_slot: Option::None,
        }
    }

    /// Add a base58 public key
    pub fn add_public_key(&mut self, base58_public_key: &'q str) -> &mut Self {
        self.base58_public_key = base58_public_key;

        self
    }

    /// Add the commitment level
    pub fn add_commitment(&mut self, commitment: &'q str) -> &mut Self {
        self.commitment = commitment;

        self
    }

    /// Add the minimum context slot
    pub fn add_min_context_slot(&mut self, min_context_slot: Option<u64>) -> &mut Self {
        self.min_context_slot = min_context_slot;

        self
    }

    /// Build the SQL query
    pub async fn query(self) -> ProxyResult<GetAccountInfoRow> {
        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let pubkey = self.base58_public_key;

        if let Some(min_context_slot) = self.min_context_slot {
            let slot = min_context_slot as i64;

            let row = pg_client
                .query_one(
                    "
                    SELECT 
                        accounts.slot,
                        accounts.data,
                        accounts.executable,
                        accounts.owner,
                        accounts.lamports,
                        accounts.rent_epoch
                    FROM accounts WHERE pubkey = '$1'
                    AND slot >= $2;",
                    &[&pubkey, &slot],
                )
                .await?;

            let outcome: GetAccountInfoRow = row.into();

            Ok(outcome)
        } else {
            let row = pg_client
                .query_one(
                    "
                    SELECT 
                        accounts.slot,
                        accounts.data,
                        accounts.executable,
                        accounts.owner,
                        accounts.lamports,
                        accounts.rent_epoch
                    FROM accounts WHERE pubkey = $1::TEXT;",
                    &[&pubkey],
                )
                .await?;

            let outcome: GetAccountInfoRow = row.into();

            Ok(outcome)
        }
    }
}

impl<'q> Default for GetAccountInfoQuery<'q> {
    fn default() -> Self {
        GetAccountInfoQuery::new()
    }
}

/// Get the current slot by querying the `MAX` slot from the database
#[derive(Debug)]
pub struct CurrentSlot {
    /// The commitment to use to get the max slot
    pub commitment: Commitment,
}

impl Default for CurrentSlot {
    fn default() -> Self {
        CurrentSlot {
            commitment: Commitment::Finalized,
        }
    }
}

impl CurrentSlot {
    /// Instantiate a new structure
    pub fn new() -> Self {
        CurrentSlot::default()
    }

    /// Change the commitment level for the query
    pub fn add_commitment(mut self, commitment: Commitment) -> Self {
        self.commitment = commitment;

        self
    }

    /// Run the query in the database and deserialize it to [Self]
    pub async fn query(self) -> ProxyResult<Context> {
        let commitment = self.commitment.queryable();

        crate::PgConnection::client_exists().await?;
        let guarded_pg_client = crate::CLIENT.read().await;
        let pg_client = guarded_pg_client.as_ref().unwrap(); // Cannot fail since `Option::None` has been handled by `PgConnection::client_exists()?;` above

        let row = pg_client
            .query_one(
                "
            SELECT MAX(slot) FROM slots WHERE status::VARCHAR = $1::TEXT;
            ",
                &[&commitment],
            )
            .await?;

        let context: Context = row.into();

        Ok(context)
    }
}
