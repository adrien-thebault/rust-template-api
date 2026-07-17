//! this gateway's own role set. Kept local to `web` rather than in
//! `rust_toolbox::axum_tools` - that crate is generic across whatever
//! services depend on it (`User::roles` is a plain `Vec<String>`), and a
//! single fixed `Role` enum living there would force every consumer onto the
//! same role set.

/// a permission an authenticated [`rust_toolbox::axum_tools::auth::User`] may
/// hold. Deliberately small today (this gateway only distinguishes "admin or
/// not") but a real enum, not a boolean, so more roles can be added without
/// changing every call site.
pub enum Role {
    Admin,
}

impl AsRef<str> for Role {
    fn as_ref(&self) -> &str {
        match self {
            Self::Admin => "ADMIN",
        }
    }
}
