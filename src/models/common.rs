use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub success: bool,
    pub msg: String,
    pub obj: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn into_result(self) -> crate::Result<Option<T>> {
        if self.success {
            Ok(self.obj)
        } else {
            Err(crate::Error::Api(self.msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_ok() {
        let raw = r#"{"success":true,"msg":"","obj":42}"#;
        let resp: ApiResponse<i32> = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.into_result().unwrap(), Some(42));
    }

    #[test]
    fn failed_response_is_err() {
        let raw = r#"{"success":false,"msg":"bad input","obj":null}"#;
        let resp: ApiResponse<i32> = serde_json::from_str(raw).unwrap();
        let err = resp.into_result().unwrap_err();
        assert!(err.to_string().contains("bad input"));
    }
}
