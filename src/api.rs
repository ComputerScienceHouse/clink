use url::Url;
use isahc::Request;
use isahc::auth::Authentication;
use isahc::prelude::*;

pub struct API {
  token: Option<String>,
}

impl API {
  pub fn new() -> API {
    return API {
      token: None
    };
  }
  pub fn get_token(self: &mut API) -> Result<String, Box<dyn std::error::Error>> {
    return match &self.token {
      Some(token) => Ok(token.to_string()),
      None => {
        let response = Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/auth?client_id=clidrink&redirect_uri=drink%3A%2F%2Fcallback&response_type=token%20id_token&scope=openid%20profile%20drink_balance&state=&nonce=")
          .authentication(Authentication::negotiate())
          .body(())?.send()?;
        println!("{:?}", response.headers());
        let location = match response.headers().get("Location") {
          Some(location) => location,
          None => panic!("Fuck!"),
        };
        let url = Url::parse(&location.to_str()?.replace("#", "?"))?;

        for (key, value) in url.query_pairs() {
          println!("Wat {} = {}", key, value);
          if key == "access_token" {
            let value = "Bearer ".to_owned() + &value.to_string();
            self.token = Some(value.to_string());
            return Ok(value.to_string());
          }
        }
        // yolo?
        panic!("Why no????");
      }
    }
  }
}
