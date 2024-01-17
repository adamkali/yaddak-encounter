use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{errors::YaddakError, user::User};

#[derive(Serialize, ToSchema)]
#[aliases(
    UserDetailedResponse = DetailedResponse<User>,
    UserListDetailedResponse = DetailedResponse<Vec<User>>,
    UuidDetailedResponse = DetailedResponse<Uuid>
)]
#[serde(rename_all="camelCase")]
pub struct DetailedResponse<T>
where T: Serialize + Default {
    pub data: Option<T>,
    pub error: Option<YaddakError>
}

impl<T> DetailedResponse<T>
where T: Serialize + Default {
    pub fn absorb_data(data: T)
    -> Self {
        Self {
            data: Some(data),
            error: None
        }
    }
    
    pub fn absorb_error(error: YaddakError)
    -> Self {
        Self {
            data: None,
            error: Some(error)
        }
    }
}
