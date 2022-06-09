use http::status::StatusCode;
use isahc::{auth::Authentication, prelude::*, HttpClient, Request};
use rpassword::read_password;
use serde_json::json;
use serde_json::{Map, Value};
use serde::Deserialize;
use std::env;
use std::fmt;
use std::io::{Read, Write};
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
  pub number: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Item {
  pub id: u64,
  pub name: String,
  pub price: u64,
}

impl std::error::Error for APIError {}

impl fmt::Display for APIError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    return f.write_str(match self {
      APIError::Unauthorized => "Unauthorized (Did your Kerberos ticket expire?: `kinit`)",
      APIError::BadFormat => "BadFormat (The server sent data we didn't understand)",
    });
  }
}

impl API {
  pub fn new() -> API {
    let mut api = API { token: None };
    api.get_token();
    return api;
  }
  pub fn drop(self: &mut API, machine: String, slot: u8) -> Result<(), Box<dyn std::error::Error>> {
    let token = self.get_token()?;

    let client = HttpClient::new()?;
    let request = Request::post("https://drink.csh.rit.edu/drinks/drop")
      .header("Content-Type", "application/json")
      .header("Authorization", token)
      .body(
        json!({
          "machine": machine,
          "slot": slot,
        })
        .to_string(),
      )?;
    let mut response = client.send(request)?;
    let body: Value = response.json()?;
    return match response.status() {
      StatusCode::OK => Ok(()),
      _ => {
        eprintln!("Couldn't drop: {}", body["error"].as_str().unwrap());
        return Err(Box::new(APIError::BadFormat));
      }
    };
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
    };
  }

  fn login() {
    // Get credentials
    let username: Option<String> = std::env::var("CLINK_USERNAME")
      .map(|it| Some(it))
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
    //let token = self.get_token()?;
    let client = HttpClient::new()?;
    // Can also be used to get other user information
    let request =
      Request::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/userinfo")
        .header("Authorization", self.get_token()?)
        .body(())?;
    let response: Value = client.send(request)?.json()?;
    let uid = response["preferred_username"].as_str().unwrap().to_string();
    let credit_request = Request::get(format!(
      "https://drink.csh.rit.edu/users/credits?uid={}",
      uid
    ))
    .header("Authorization", self.get_token()?)
    .body(())?;
    let credit_response: Value = client.send(credit_request)?.json()?;
    Ok(
      credit_response["user"]["drinkBalance"]
        .as_str()
        .unwrap()
        .parse::<u64>()?,
    ) // Coffee
  }

  pub fn get_machine_status(self: &mut API) -> Result<DrinkList, Box<dyn std::error::Error>> {
    self.get_status_for_machine(None)
  }

  pub fn get_status_for_machine(self: &mut API, machine: Option<&str>) -> Result<DrinkList, Box<dyn std::error::Error>> {
    let token = self.get_token()?;
    let client = HttpClient::new()?;
    let request = Request::get(
      format!("https://drink.csh.rit.edu/drinks{}", match machine {
        Some(machine) => format!("?machine={}", machine),
        None => "".to_string(),
      }))
      .header("Authorization", token)
      .body(())?;
    Ok(client.send(request)?.json::<DrinkList>()?)
  }
}
