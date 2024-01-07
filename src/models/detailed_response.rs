use serde::Serialize;

use super::errors::YaddakError;

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct DetailedResponse<T>
where T: Serialize + Default {
    #[serde(skip_serializing_if="Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if="Option::is_none")]
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
