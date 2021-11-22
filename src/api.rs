use std::fmt;
use url::Url;
use isahc::{Request, auth::Authentication, prelude::*, HttpClient};
use http::status::StatusCode;
use serde_json::json;

pub struct API {
  token: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
  Unauthorized,
  BadFormat
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    return f.write_str(match self {
      Unauthorized => "Unauthorized (Did your Kerberos ticket expire?: `kinit`)",
      BadFormat => "BadFormat (The server sent data we didn't understand)"
    });
  }
}

impl API {
  pub fn new() -> API {
    return API {
      token: None
    };
  }
  pub fn drop(self: &mut API, machine: String, slot: u8) -> Result<(), Box<dyn std::error::Error>> {
    let token = self.get_token()?;

    let client = HttpClient::new()?;
    let request = Request::post("https://drink.csh.rit.edu/drinks/drop")
      .header("Authorization", token)
      .body(json!({
        "machine": machine,
        "slot": slot,
      }).to_string())?;
    let response = client.send(request)?;
    return match response.status() {
      StatusCode::OK => Ok(()),
      _ => Err(Box::new(APIError::BadFormat))
    }
  }
  pub fn get_token(self: &mut API) -> Result<String, Box<dyn std::error::Error>> {
    return match &self.token {
      Some(token) => Ok(token.to_string()),
      None => {
        let response = Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/auth?client_id=clidrink&redirect_uri=drink%3A%2F%2Fcallback&response_type=token%20id_token&scope=openid%20profile%20drink_balance&state=&nonce=")
          .authentication(Authentication::negotiate())
          .body(())?.send()?;
        let location = match response.headers().get("Location") {
          Some(location) => location,
          None => return Err(Box::new(APIError::Unauthorized)),
        };
        let url = Url::parse(&location.to_str()?.replace("#", "?"))?;

        for (key, value) in url.query_pairs() {
          if key == "access_token" {
            let value = "Bearer ".to_owned() + &value.to_string();
            self.token = Some(value.to_string());
            return Ok(value.to_string());
          }
        }
        return Err(Box::new(APIError::BadFormat));
      }
    }
  }
}
