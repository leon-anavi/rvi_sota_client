use chan::Sender;
use http_client::{HttpClient, HttpRequest, HttpResponse};
use std::cell::RefCell;

use datatype::Error;


pub struct TestHttpClient {
    replies: RefCell<Vec<String>>
}

impl TestHttpClient {
    pub fn new() -> TestHttpClient {
        TestHttpClient { replies: RefCell::new(Vec::new()) }
    }

    pub fn from(replies: Vec<String>) -> TestHttpClient {
        TestHttpClient { replies: RefCell::new(replies) }
    }
}

impl HttpClient for TestHttpClient {
    fn chan_request(&self, req: HttpRequest, resp_tx: Sender<HttpResponse>) {
        match self.replies.borrow_mut().pop() {
            Some(body) => resp_tx.send(Ok(body.as_bytes().to_vec())),
            None       => resp_tx.send(Err(Error::ClientError(req.url.to_string())))
        }
    }

    fn is_testing(&self) -> bool { true }
}
