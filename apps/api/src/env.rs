use tronc::env::{Core, EnvError, Vars};

/// Everything the binary reads from its environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// The chassis variables: port, database, proxies, Journal.
    pub core: Core,
    /// Registre sign-in. `sso_only` is always true.
    pub porte: porte::Config,
    /// `PORTE_TOKEN_KEY`: seals the stored OIDC tokens.
    pub token_key: String,
    /// `MCP_RESOURCE`: the public URL of `/mcp`, read now so a deploy can set it before `/mcp` ships.
    pub mcp_resource: String,
    /// `MCP_STATE_KEY`: the HMAC key of the MCP confirmation flow.
    pub mcp_state_key: String,
    /// `<APP>_URL` and `<APP>_CLIENT_ID` of every suite app Toile calls.
    pub apps: crate::modules::suite::Apps,
}

impl Config {
    /// Reads the process environment.
    pub fn load() -> Result<Self, EnvError> {
        Self::from_vars(&Vars::process())
    }

    /// Reads `vars`, refusing a production boot without Registre.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let core = Core::from_vars(vars, true)?;
        let text = |key: &str| vars.string(key, "").trim().to_owned();
        let porte = porte::Config { app_name: "Toile".to_owned(), sso_only: true, ..porte::Config::from_vars(vars)? };
        let config = Config {
            porte,
            token_key: text("PORTE_TOKEN_KEY"),
            mcp_resource: text("MCP_RESOURCE"),
            mcp_state_key: text("MCP_STATE_KEY"),
            apps: crate::modules::suite::Apps::from_vars(vars),
            core,
        };
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result<(), EnvError> {
        if self.core.is_production() && self.porte.issuer.is_empty() {
            return Err(EnvError("OIDC_ISSUER is required in production".to_owned()));
        }
        if self.core.is_production() && self.token_key.is_empty() {
            return Err(EnvError("PORTE_TOKEN_KEY is required in production".to_owned()));
        }
        if !self.mcp_resource.is_empty() && self.mcp_state_key.is_empty() {
            return Err(EnvError("MCP_STATE_KEY is required with MCP_RESOURCE".to_owned()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(extra: &[(&str, &str)]) -> Vars {
        let mut pairs = vec![("DATABASE_URL", "postgres://x/y")];
        pairs.extend_from_slice(extra);
        Vars::from_pairs(pairs)
    }

    #[test]
    fn production_needs_the_issuer() {
        let error = Config::from_vars(&vars(&[("APP_ENV", "production")])).unwrap_err();
        assert!(error.0.contains("OIDC_ISSUER"), "{error}");
    }

    #[test]
    fn mcp_needs_a_state_key() {
        let error = Config::from_vars(&vars(&[("MCP_RESOURCE", "https://toile.test/mcp")])).unwrap_err();
        assert!(error.0.contains("MCP_STATE_KEY"), "{error}");
    }

    #[test]
    fn suite_apps_default_to_their_facile_host_and_stay_off_without_a_client_id() {
        let config =
            Config::from_vars(&vars(&[("NUAGE_CLIENT_ID", "nuage-id"), ("OPUS_URL", "https://o.test/")])).unwrap();
        assert_eq!(config.apps.nuage.url, "https://nuage.facile.studio");
        assert!(config.apps.nuage.enabled() && !config.apps.docs.enabled());
        assert_eq!(config.apps.opus.url, "https://o.test");
        assert_eq!(config.apps.nuage.target(true).scopes, ["nuage.read", "nuage.write"]);
    }

    #[test]
    fn defaults_are_sso_only_under_the_toile_name() {
        let config = Config::from_vars(&vars(&[])).unwrap();
        assert!(config.porte.sso_only);
        assert_eq!(config.porte.app_name, "Toile");
    }
}
