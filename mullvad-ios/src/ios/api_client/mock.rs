use mockito::{Mock, ServerGuard};

#[derive(uniffi::Object)]
pub struct ServerMock {
    #[expect(unused)]
    server: ServerGuard,
    #[expect(unused)]
    mocks: Vec<Mock>,
    port: u16,
}
impl ServerMock {
    pub fn new(server: ServerGuard, mocks: Vec<Mock>) -> Self {
        Self {
            port: server.socket_address().port(),
            server,
            mocks,
        }
    }
}
#[uniffi::export]
impl ServerMock {
    pub fn port(&self) -> u16 {
        self.port
    }

    #[uniffi::constructor]
    pub fn get(endpoints: Vec<MockEndpoint>) -> ServerMock {
        let mut server = mockito::Server::new();
        let mocks = endpoints
            .into_iter()
            .map(|endpoint| {
                server
                    .mock("GET", &*endpoint.path)
                    .with_header("content-type", "application/json")
                    .with_status(endpoint.response_code as usize)
                    .with_body(&*endpoint.response_body)
                    .create()
            })
            .collect();

        Self::new(server, mocks)
    }

    #[uniffi::constructor]
    pub fn post(path: String, response_code: u64, match_body: String) -> ServerMock {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("POST", &*path)
            .with_header("content-type", "application/json")
            .with_status(response_code as usize)
            .match_body(mockito::Matcher::JsonString(match_body))
            .create();
        Self::new(server, vec![mock])
    }
}

#[derive(uniffi::Record)]
pub struct MockEndpoint {
    path: String,
    response_code: u64,
    response_body: String,
}
