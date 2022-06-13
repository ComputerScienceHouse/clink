use http::status::StatusCode;
use http::Uri;
use isahc::{auth::Authentication, prelude::*, HttpClient, Request};
use rpassword::read_password;
use serde::{de, Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use url::Url;
use users::get_current_username;

pub struct API {
  token: Option<String>,
}

#[derive(Debug)]
pub enum APIError {
  Unauthorized,
  BadFormat,
  ServerError(Option<Uri>, String),
}

#[derive(Deserialize, Debug, Clone)]
struct ErrorResponse {
  error: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DrinkList {
  pub machines: Vec<Machine>,
  pub message: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Machine {
  pub display_name: String,
  pub id: u64,
  pub is_online: bool,
  pub name: String,
  pub slots: Vec<Slot>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Slot {
  pub active: bool,
  pub count: Option<u64>,
  pub empty: bool,
  pub item: Item,
  pub machine: u64,
  pub number: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
  pub id: u64,
  pub name: String,
  pub price: u64,
}

#[derive(Deserialize, Debug, Clone)]
struct User {
  preferred_username: String,
}

#[derive(Deserialize, Debug, Clone)]
struct CreditResponse {
  user: CreditUser,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
struct CreditUser {
  #[serde(deserialize_with = "number_string_deserializer")]
  drinkBalance: u64,
}

fn number_string_deserializer<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: de::Deserializer<'de>,
{
  let number: String = Deserialize::deserialize(deserializer)?;
  match number.parse::<u64>() {
    Ok(res) => Ok(res),
    Err(e) => Err(de::Error::custom(format!(
      "Failed to deserialize u64: {}",
      e
    ))),
  }
}

#[derive(Serialize, Debug, Clone)]
struct DropRequest {
  machine: String,
  slot: u8,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
struct DropResponse {
  #[serde(deserialize_with = "number_string_deserializer")]
  drinkBalance: u64,
  // message: String,
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      APIError::Unauthorized => write!(
        f,
        "Unauthorized (Did your Kerberos ticket expire?: `kinit`)"
      ),
      APIError::BadFormat => write!(f, "BadFormat (The server sent data we didn't understand)"),
      APIError::ServerError(path, message) => write!(
        f,
        "ServerError for {}: {}",
        match path {
          Some(ref uri) => uri.to_string(),
          None => "<unknown>".to_string(),
        },
        message
      ),
    }
  }
}

impl Default for API {
  fn default() -> Self {
    Self::new()
  }
}

enum APIBody<T: Serialize> {
  Json(T),
  NoBody,
}

impl<T: Serialize> From<APIBody<T>> for isahc::Body {
  fn from(body: APIBody<T>) -> Self {
    match body {
      APIBody::Json(value) => serde_json::to_string(&value).unwrap().into(),
      APIBody::NoBody => ().into(),
    }
  }
}

impl API {
  pub fn new() -> API {
    let mut api = API { token: None };
    api.get_token().ok();
    api
  }
  fn authenticated_request<O, I>(
    self: &mut API,
    builder: http::request::Builder,
    input: APIBody<I>,
  ) -> Result<O, Box<dyn std::error::Error>>
  where
    I: Serialize,
    O: de::DeserializeOwned,
  {
    let client = HttpClient::new()?;
    let token = self.get_token()?;
    let builder = builder
      .header("Authorization", token)
      .header("Accept", "application/json");
    let builder = match input {
      APIBody::Json(_) => builder.header("Content-Type", "application/json"),
      APIBody::NoBody => builder,
    };
    let mut response = client.send(builder.body(input)?)?;
    match response.status() {
      StatusCode::OK => match response.json::<O>() {
        Ok(value) => Ok(value),
        Err(_) => Err(Box::new(APIError::BadFormat)),
      },
      _ => {
        let text = response.text()?;
        Err(Box::new(APIError::ServerError(
          response.effective_uri().cloned(),
          serde_json::from_str::<ErrorResponse>(&text)
            .map(|body| body.error)
            .unwrap_or(text),
        )))
      }
    }
  }
  pub fn drop(
    self: &mut API,
    machine: String,
    slot: u8,
  ) -> Result<u64, Box<dyn std::error::Error>> {
    self
      .authenticated_request::<DropResponse, _>(
        Request::post("https://drink.csh.rit.edu/drinks/drop"),
        APIBody::Json(DropRequest { machine, slot }),
      )
      .map(|drop| drop.drinkBalance)
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
          None => {
            API::login();
            return self.get_token();
          }
        };
        let url = Url::parse(&location.to_str()?.replace('#', "?"))?;

        for (key, value) in url.query_pairs() {
          if key == "access_token" {
            let value = format!("Bearer {}", value);
            self.token = Some(value.clone());
            return Ok(value);
          }
        }
        return Err(Box::new(APIError::BadFormat));
      }
    };
  }

  fn login() {
    // Get credentials
    let username: Option<String> = std::env::var("CLINK_USERNAME")
      .map(Some)
      .unwrap_or_else(|_| get_current_username().and_then(|it| it.into_string().ok()));

    let username: String = username.unwrap();

    // Start kinit, ready to get password from pipe
    let mut process = Command::new("kinit")
      .arg(format!("{}@CSH.RIT.EDU", username))
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .spawn()
      .unwrap();

    // Get password
    println!("Please enter password for {}: ", username);
    let password = read_password().unwrap();

    // Pipe password into the prompt that "comes up"
    process
      .stdin
      .as_ref()
      .unwrap()
      .write_all(password.as_bytes())
      .unwrap();

    // Wait for login to be complete before continuting
    process.wait().unwrap();

    println!("...\n\n");
  }

  pub fn get_credits(self: &mut API) -> Result<u64, Box<dyn std::error::Error>> {
    // Can also be used to get other user information
    let user: User = self.authenticated_request(
      Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/userinfo"),
      APIBody::NoBody as APIBody<serde_json::Value>,
    )?;
    let credit_response: CreditResponse = self.authenticated_request(
      Request::get(format!(
        "https://drink.csh.rit.edu/users/credits?uid={}",
        user.preferred_username
      )),
      APIBody::NoBody as APIBody<serde_json::Value>,
    )?;
    Ok(credit_response.user.drinkBalance)
  }

  pub fn get_machine_status(self: &mut API) -> Result<DrinkList, Box<dyn std::error::Error>> {
    self.get_status_for_machine(None)
  }

  pub fn get_status_for_machine(
    self: &mut API,
    machine: Option<&str>,
  ) -> Result<DrinkList, Box<dyn std::error::Error>> {
    self.authenticated_request(
      Request::get(format!(
        "https://drink.csh.rit.edu/drinks{}",
        match machine {
          Some(machine) => format!("?machine={}", machine),
          None => "".to_string(),
        }
      )),
      APIBody::NoBody as APIBody<serde_json::Value>,
    )
  }
}
