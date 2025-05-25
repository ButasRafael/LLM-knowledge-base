use crate::error::{Error, Result};
use crate::web::mw_auth::CtxExtError::CtxCannotNewRootCtx;

#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: i64,
    role: String,
}

impl Ctx {
    pub fn root_ctx() -> Self {
        Ctx {
            user_id: 0,
            role: "root".to_string(),
        }
    }

    pub fn new(user_id: i64, role: &str) -> Result<Self> {
        if user_id == 0 {
            return Err(Error::CtxExt(CtxCannotNewRootCtx));
        }
        Ok(Self {
            user_id,
            role: role.to_string(),
        })
    }
    // Accessors
    pub fn user_id(&self) -> i64 {
        self.user_id
    }
    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }
}
