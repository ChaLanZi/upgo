/// Account settings page.
///
/// Sections:
/// - Password Change: old password → new password
/// - Email Change: new email → verification → confirm
/// - Sessions: list active sessions, logout single/all
/// - Delete Account: confirmation code → soft delete
pub struct SettingsPage {
    // Password change
    pub old_password: String,
    pub new_password: String,
    pub password_error: Option<String>,
    pub password_loading: bool,
    pub on_change_password: Option<Box<dyn Fn(String, String)>>,

    // Email change
    pub new_email: String,
    pub email_verification_code: String,
    pub email_code_sent: bool,
    pub email_error: Option<String>,
    pub email_loading: bool,
    pub on_change_email: Option<Box<dyn Fn(String)>>,
    pub on_confirm_email: Option<Box<dyn Fn(String)>>,

    // Delete account
    pub delete_code: String,
    pub delete_code_sent: bool,
    pub delete_error: Option<String>,
    pub delete_loading: bool,
    pub on_delete_request: Option<Box<dyn Fn()>>,
    pub on_delete_confirm: Option<Box<dyn Fn(String)>>,
    pub on_delete_cancel: Option<Box<dyn Fn()>>,

    // Sessions
    pub sessions: Vec<SessionInfo>,
    pub on_logout_all: Option<Box<dyn Fn()>>,
}

pub struct SessionInfo {
    pub session_id: String,
    pub platform: String,
    pub created_at: String,
    pub is_current: bool,
}

impl SettingsPage {
    pub fn new() -> Self {
        Self {
            old_password: String::new(),
            new_password: String::new(),
            password_error: None,
            password_loading: false,
            on_change_password: None,
            new_email: String::new(),
            email_verification_code: String::new(),
            email_code_sent: false,
            email_error: None,
            email_loading: false,
            on_change_email: None,
            on_confirm_email: None,
            delete_code: String::new(),
            delete_code_sent: false,
            delete_error: None,
            delete_loading: false,
            on_delete_request: None,
            on_delete_confirm: None,
            on_delete_cancel: None,
            sessions: Vec::new(),
            on_logout_all: None,
        }
    }

    pub fn change_password(&mut self) {
        if self.new_password.len() < 8 {
            self.password_error = Some("New password must be at least 8 characters".to_string());
            return;
        }
        if let Some(ref cb) = self.on_change_password {
            cb(self.old_password.clone(), self.new_password.clone());
        }
    }

    pub fn request_email_change(&mut self) {
        if !self.new_email.contains('@') {
            self.email_error = Some("Invalid email format".to_string());
            return;
        }
        if let Some(ref cb) = self.on_change_email {
            cb(self.new_email.clone());
        }
    }

    pub fn confirm_email_change(&mut self) {
        if let Some(ref cb) = self.on_confirm_email {
            cb(self.email_verification_code.clone());
        }
    }

    pub fn request_delete(&mut self) {
        if let Some(ref cb) = self.on_delete_request {
            cb();
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Some(ref cb) = self.on_delete_confirm {
            cb(self.delete_code.clone());
        }
    }

    pub fn cancel_delete(&mut self) {
        if let Some(ref cb) = self.on_delete_cancel {
            cb();
        }
    }
}
