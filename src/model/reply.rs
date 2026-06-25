#[derive(Clone, Debug)]
pub struct Reply {
    pub text: String,
    pub name: String,
    pub replied_at: String,
}

pub use crate::helper::consts::MAX_REPLY_LENGTH as MAX_LENGTH;
