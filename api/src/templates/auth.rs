/// Which tab to activate by default on the auth page.
#[derive(Clone, Copy, Default)]
pub enum AuthTab {
    #[default]
    Login,
    Register,
    Reset,
}

impl AuthTab {
    pub fn from_query(tab: Option<&str>) -> Self {
        match tab {
            Some("login") => AuthTab::Login,
            Some("register") => AuthTab::Register,
            Some("reset") => AuthTab::Reset,
            _ => AuthTab::default(),
        }
    }
}
