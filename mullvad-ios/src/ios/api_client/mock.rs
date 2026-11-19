use mockito::{Mock, ServerGuard};

#[derive(uniffi::Object)]
pub struct MockServer {
    #[expect(unused)]
    server: ServerGuard,
    #[expect(unused)]
    mocks: Vec<Mock>,
    port: u16,
}
#[uniffi::export]
impl MockServer {
    pub fn port(&self) -> u16 {
        self.port
    }

    #[uniffi::constructor(default(get = [], post = []))]
    pub fn new(get: Vec<MockEndpoint>, post: Vec<MockEndpoint>) -> Self {
        let mut server = mockito::Server::new();

        let mut mocks = Vec::with_capacity(get.len() + post.len());

        mocks.extend(get.into_iter().map(|endpoint| {
            server
                .mock("GET", &*endpoint.path)
                .with_header("content-type", "application/json")
                .with_status(endpoint.response_code as usize)
                .with_body(&*endpoint.response_body)
                .create()
        }));
        mocks.extend(post.into_iter().map(|endpoint| {
            server
                .mock("POST", &*endpoint.path)
                .with_header("content-type", "application/json")
                .with_status(endpoint.response_code as usize)
                .match_body(mockito::Matcher::JsonString(endpoint.response_body))
                .create()
        }));

        Self {
            port: server.socket_address().port(),
            server,
            mocks,
        }
    }
}

#[derive(uniffi::Record)]
pub struct MockEndpoint {
    path: String,
    response_code: u64,
    response_body: String,
}
