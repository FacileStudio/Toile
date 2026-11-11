use porte::exchange::Target;
use tronc::env::Vars;

/// One suite app Toile reaches for a person: its public origin and its Registre client id,
/// the audience of the tokens exchanged for it. An empty audience turns the integration off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct App {
    /// The app's key: `nuage`, `opus`, `docs`, `feuilles` or `agenda`, the prefix of its scopes.
    pub key: &'static str,
    /// The public origin, without a trailing slash.
    pub url: String,
    /// The app's Registre client id.
    pub audience: String,
}

impl App {
    fn from_vars(vars: &Vars, key: &'static str) -> Self {
        let upper = key.to_ascii_uppercase();
        let fallback = format!("https://{key}.facile.studio");
        let url = vars.string(&format!("{upper}_URL"), &fallback).trim().trim_end_matches('/').to_owned();
        let audience = vars.string(&format!("{upper}_CLIENT_ID"), "").trim().to_owned();
        App { key, url, audience }
    }

    /// Whether Toile may call this app.
    pub fn enabled(&self) -> bool {
        !self.audience.is_empty()
    }

    /// What a token for this app is exchanged for: `<key>.read`, plus `<key>.write` when `write`.
    pub fn target(&self, write: bool) -> Target {
        let mut scopes = vec![format!("{}.read", self.key)];
        if write {
            scopes.push(format!("{}.write", self.key));
        }
        Target::new(&self.audience, scopes)
    }

    /// Whether `url` points into this app: same scheme, host and port.
    pub fn owns(&self, url: &url::Url) -> bool {
        url::Url::parse(&self.url).is_ok_and(|own| own.origin() == url.origin())
    }
}

/// The suite apps Toile reads and writes for a person.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Apps {
    /// Image uploads, in the person's Toile app folder.
    pub nuage: App,
    /// Tasks.
    pub opus: App,
    /// Documents.
    pub docs: App,
    /// Workbooks.
    pub feuilles: App,
    /// Events.
    pub agenda: App,
}

impl Apps {
    /// Reads `<APP>_URL` (default `https://<app>.facile.studio`) and `<APP>_CLIENT_ID`.
    pub fn from_vars(vars: &Vars) -> Self {
        Apps {
            nuage: App::from_vars(vars, "nuage"),
            opus: App::from_vars(vars, "opus"),
            docs: App::from_vars(vars, "docs"),
            feuilles: App::from_vars(vars, "feuilles"),
            agenda: App::from_vars(vars, "agenda"),
        }
    }
}
