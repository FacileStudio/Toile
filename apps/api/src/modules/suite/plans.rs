use std::sync::Arc;

use porte::directory::Snapshot;
use porte::forfait::{Feature, ImportError, Scope, Store};
use sqlx::PgPool;

/// `ai.access`: AI features, off on the free plan.
pub const AI_ACCESS: &str = "ai.access";
/// `toile.upload_mb`: the largest image one upload may be, in MB; 10 on the free plan.
pub const UPLOAD_MB: &str = "toile.upload_mb";

const DEFAULTS: &[u8] = br#"{"version":1,"snapshot_version":1,"entitlements_revision":1,"spaces":{},"people":{}}"#;

/// The plan values Registre serves for Toile, kept in the one `entitlement_state` row so a
/// restart enforces the last known plans instead of failing open again.
#[derive(Clone)]
pub struct Plans {
    store: Arc<Store>,
    pool: PgPool,
}

#[derive(serde::Deserialize)]
struct Head {
    entitlements_revision: i64,
}

impl Plans {
    /// The plans for `issuer` (the base of every upgrade link), failing open until a restore or
    /// an apply hands it an authoritative snapshot.
    pub fn new(pool: PgPool, issuer: &str) -> Self {
        let features = [Feature::bool(AI_ACCESS, false), Feature::limit(UPLOAD_MB, 10)];
        Plans { store: Arc::new(Store::new(features).with_issuer(issuer)), pool }
    }

    fn enforcing(&self) -> bool {
        serde_json::from_slice::<Head>(&self.store.export()).is_ok_and(|head| head.entitlements_revision > 0)
    }

    /// Loads the saved state; run it before the first directory pull. No row keeps failing open.
    /// A row that cannot be imported proves plans were enforced once, so the defaults are
    /// enforced until Registre answers.
    pub async fn restore(&self) -> Result<(), sqlx::Error> {
        let row: Option<String> = sqlx::query_scalar("SELECT document FROM entitlement_state WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;
        let Some(document) = row else { return Ok(()) };
        let imported = self.store.import(document.as_bytes());
        if self.enforcing() || matches!(imported, Err(ImportError::AlreadyApplied)) {
            return Ok(());
        }
        tracing::error!(error = ?imported.err(), "forfait: saved entitlements unreadable; enforcing the defaults");
        _ = self.store.import(DEFAULTS);
        Ok(())
    }

    /// Hands `snapshot` to the store and saves the result. A store still failing open saves
    /// nothing.
    pub async fn apply(&self, snapshot: &Snapshot) -> Result<(), sqlx::Error> {
        self.store.apply(snapshot);
        if !self.enforcing() {
            return Ok(());
        }
        let document = String::from_utf8_lossy(&self.store.export()).into_owned();
        sqlx::query(
            "INSERT INTO entitlement_state (id, document) VALUES (1, $1)
             ON CONFLICT (id) DO UPDATE SET document = EXCLUDED.document, updated_at = now()",
        )
        .bind(document)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Whether `key` is on for `scope`.
    pub fn enabled(&self, scope: &Scope, key: &str) -> bool {
        self.store.enabled(scope, key)
    }

    /// The ceiling of `key` for `scope`, -1 meaning unlimited.
    pub fn limit(&self, scope: &Scope, key: &str) -> i64 {
        self.store.limit(scope, key)
    }
}
