use rustc_serialize::json;

use datatype::{AccessToken, Error, Method, Url};
use http_client::{HttpClient, HttpRequest};


pub fn authenticate(server: Url, client: &HttpClient) -> Result<AccessToken, Error> {
    debug!("authenticate()");
    let resp_rx = client.send_request(HttpRequest{ method: Method::Post, url: server, body: None });
    let resp    = resp_rx.recv().expect("no authenticate response received");
    let data    = try!(resp);
    let body    = try!(String::from_utf8(data));
    debug!("authenticate, body: `{}`", body);
    Ok(try!(json::decode(&body)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use datatype::{AccessToken, Url};
    use http_client::TestHttpClient;


    fn test_server() -> Url {
        Url::parse("http://localhost:8000").unwrap()
    }

    #[test]
    fn test_authenticate() {
        let token  = r#"{"access_token": "token", "token_type": "type", "expires_in": 10, "scope": ["scope"]}"#;
        let client = TestHttpClient::from(vec![token.to_string()]);
        let expect = AccessToken {
            access_token: "token".to_string(),
            token_type:   "type".to_string(),
            expires_in:   10,
            scope:        vec!["scope".to_string()]
        };
        assert_eq!(expect, authenticate(test_server(), &client).unwrap());
    }

    #[test]
    fn test_authenticate_no_token() {
        let client = TestHttpClient::from(vec!["".to_string()]);
        // XXX: Old error message was arguably a lot better...
        // "Authentication error, didn't receive access token.")
        let expect = r#"Failed to decode JSON: ParseError(SyntaxError("EOF While parsing value", 1, 1))"#;
        assert_eq!(expect, format!("{}", authenticate(test_server(), &client).unwrap_err()));
    }

    #[test]
    fn test_authenticate_bad_json() {
        let client = TestHttpClient::from(vec![r#"{"apa": 1}"#.to_string()]);
        let expect = r#"Failed to decode JSON: MissingFieldError("access_token")"#;
        assert_eq!(expect, format!("{}", authenticate(test_server(), &client).unwrap_err()));
    }
}
