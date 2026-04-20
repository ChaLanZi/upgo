/// Registration page component with email verification flow.
///
/// Two-step flow:
/// 1. Submit email + password + nickname → receive verification code
/// 2. Enter verification code → complete registration → auto-login
pub struct RegisterPage {
    // Step 1 fields
    pub email: String,
    pub password: String,
    pub nickname: String,
    // Step 2 fields
    pub verification_code: String,
    pub code_sent: bool,
    // State
    pub error: Option<String>,
    pub loading: bool,
    pub on_register: Option<Box<dyn Fn(String, String, String)>>,  // email, password, nickname
    pub on_verify: Option<Box<dyn Fn(String, String)>>,  // email, code
    pub on_login_click: Option<Box<dyn Fn()>>,
}

impl RegisterPage {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            nickname: String::new(),
            verification_code: String::new(),
            code_sent: false,
            error: None,
            loading: false,
            on_register: None,
            on_verify: None,
            on_login_click: None,
        }
    }

    pub fn submit_registration(&mut self) {
        if self.email.is_empty() || !self.email.contains('@') {
            self.error = Some("Valid email is required".to_string());
            return;
        }
        if self.password.len() < 8 {
            self.error = Some("Password must be at least 8 characters".to_string());
            return;
        }
        if self.nickname.is_empty() {
            self.error = Some("Nickname is required".to_string());
            return;
        }
        if let Some(ref cb) = self.on_register {
            cb(self.email.clone(), self.password.clone(), self.nickname.clone());
        }
    }

    pub fn submit_verification(&mut self) {
        if self.verification_code.len() != 6 {
            self.error = Some("Verification code must be 6 digits".to_string());
            return;
        }
        if let Some(ref cb) = self.on_verify {
            cb(self.email.clone(), self.verification_code.clone());
        }
    }
}

impl Default for RegisterPage {
    fn default() -> Self {
        Self::new()
    }
}
