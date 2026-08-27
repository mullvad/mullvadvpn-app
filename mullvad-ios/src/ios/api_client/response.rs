use mullvad_api::{
    RelayListProxy, StatusCode,
    rest::{self, Response},
};

#[derive(uniffi::Object)]
pub struct ApiResponse {
    body: Option<Vec<u8>>,
    etag: Option<String>,
    status_code: u16,
    error_description: Option<String>,
    server_response_code: Option<String>,
    success: bool,
}
#[uniffi::export]
impl ApiResponse {
    pub fn body(&self) -> Option<Vec<u8>> {
        self.body.clone()
    }
    pub fn etag(&self) -> Option<String> {
        self.etag.clone()
    }
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
    pub fn error_description(&self) -> Option<String> {
        self.error_description.clone()
    }
    pub fn server_response_code(&self) -> Option<String> {
        self.server_response_code.clone()
    }
    pub fn success(&self) -> bool {
        self.success
    }
}
impl ApiResponse {
    pub async fn with_body(response: Response<hyper::body::Incoming>) -> Result<Self, rest::Error> {
        let maybe_etag = RelayListProxy::extract_etag(&response);

        let status_code: u16 = response.status().into();
        let body: Vec<u8> = response.body().await?;

        let etag = match maybe_etag {
            Some(etag) => Some(etag.0),
            None => None,
        };

        Ok(Self {
            body: Some(body),
            etag,
            status_code,
            error_description: None,
            server_response_code: None,
            success: true,
        })
    }

    pub fn ok() -> Self {
        Self {
            success: true,
            error_description: None,
            body: None,
            etag: None,
            status_code: StatusCode::NO_CONTENT.as_u16(),
            server_response_code: None,
        }
    }

    pub fn access_method_error(err: mullvad_api::access_mode::Error) -> Self {
        let error_description = err.to_string();

        Self {
            body: None,
            etag: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            error_description: Some(error_description),
            server_response_code: None,
            success: false,
        }
    }

    pub fn rest_error(err: mullvad_api::rest::Error) -> Self {
        if err.is_aborted() {
            return Self::cancelled();
        }

        let error_description = err.to_string();
        let (status_code, server_response_code): (u16, _) =
            if let rest::Error::ApiError(status_code, error_code) = err {
                (status_code.into(), Some(error_code))
            } else {
                (0, None)
            };

        Self {
            body: None,
            etag: None,
            status_code,
            error_description: Some(error_description),
            server_response_code,
            success: false,
        }
    }

    pub fn cancelled() -> Self {
        Self {
            success: false,
            error_description: Some("Request was cancelled".to_string()),
            body: None,
            etag: None,
            status_code: 0,
            server_response_code: None,
        }
    }

    pub fn other<S: Into<String>>(error: S) -> Self {
        Self {
            success: false,
            error_description: Some(error.into()),
            body: None,
            etag: None,
            status_code: 0,
            server_response_code: None,
        }
    }

    pub fn no_tokio_runtime() -> Self {
        Self {
            success: false,
            error_description: Some("Failed to get Tokio runtime".to_string()),
            body: None,
            etag: None,
            status_code: 0,
            server_response_code: None,
        }
    }
}
