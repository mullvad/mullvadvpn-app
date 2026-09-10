use mullvad_api::{
    RelayListProxy, StatusCode,
    rest::{self, Response},
};

#[derive(uniffi::Enum)]
pub enum ApiResponse {
    Body {
        body: Vec<u8>,
        etag: Option<String>,
        status_code: u16,
    },
    Error {
        status_code: u16,
        error_description: Option<String>,
        server_response_code: Option<String>,
    },
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

        Ok(Self::Body {
            body,
            etag,
            status_code,
        })
    }

    pub fn ok() -> Self {
        Self::Body {
            body: Vec::new(),
            etag: None,
            status_code: StatusCode::NO_CONTENT.as_u16(),
        }
    }

    pub fn access_method_error(err: mullvad_api::access_mode::Error) -> Self {
        let error_description = err.to_string();

        Self::Error {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            error_description: Some(error_description),
            server_response_code: None,
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

        Self::Error {
            status_code,
            error_description: Some(error_description),
            server_response_code,
        }
    }

    pub fn cancelled() -> Self {
        Self::Error {
            error_description: Some("Request was cancelled".to_string()),
            status_code: 0,
            server_response_code: None,
        }
    }

    pub fn bad_public_key_size() -> Self {
        Self::Error {
            error_description: Some("bad public key size".to_string()),
            status_code: 0,
            server_response_code: None,
        }
    }

    pub fn no_tokio_runtime() -> Self {
        Self::Error {
            error_description: Some("Failed to get Tokio runtime".to_string()),
            status_code: 0,
            server_response_code: None,
        }
    }
}
