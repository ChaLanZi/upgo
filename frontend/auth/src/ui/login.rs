/// Login page component.
///
/// Platform-agnostic structure. Adapt rendering per platform:
/// - Web: HTML form via Dioxus Web
/// - Desktop: Native window via Dioxus Desktop
/// - Mobile: Native form via Dioxus Mobile
///
/// # State
/// - `email`: String — email input
/// - `password`: String — password input
/// - `error`: Option<String> — error message
/// - `loading`: bool — submission in progress
pub struct LoginPage {
    pub email: String,
    pub password: String,
    pub error: Option<String>,
    pub loading: bool,
    pub on_login: Option<Box<dyn Fn(String, String)>>,  // email, password
    pub on_register_click: Option<Box<dyn Fn()>>,
}

impl LoginPage {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            error: None,
            loading: false,
            on_login: None,
            on_register_click: None,
        }
    }

    /// Validate form inputs
    pub fn validate(&self) -> Option<String> {
        if self.email.is_empty() {
            return Some("Email is required".to_string());
        }
        if !self.email.contains('@') {
            return Some("Invalid email format".to_string());
        }
        if self.password.len() < 8 {
            return Some("Password must be at least 8 characters".to_string());
        }
        None
    }

    /// Submit login form
    pub fn submit(&mut self) {
        if let Some(err) = self.validate() {
            self.error = Some(err);
            return;
        }
        if let Some(ref cb) = self.on_login {
            cb(self.email.clone(), self.password.clone());
        }
    }
}

impl Default for LoginPage {
    fn default() -> Self {
        Self::new()
    }
}
