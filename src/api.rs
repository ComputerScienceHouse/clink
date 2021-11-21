struct API {
  Optional<String> token;
}

impl API {
  fn new() -> API {
    return API {
      token: None
    };
  }
  fn get_token(API self) -> Result<String, ()> {
    return match self.token {
      Some(token) => token,
      None => {
        let mut response = isahc::get("https://sso.csh.rit.edu/auth/realms/csh/protocol/openid-connect/auth?"
                   "client_id=clidrink&"
                   "redirect_uri=drink%3A%2F%2Fcallback&"
                   "response_type=token%20id_token&"
                   "scope=openid%20profile%20drink_balance&"
                   "state=&nonce=")?
        let location = response.headers().get("Location")?;
        let hashIndex = 1 + location.find("#")?;
        let mut params = HashMap::new();
        let mut keyIndex = 0;
        let mut keyLength = 
        let mut valIndex = 0;
        for i in hashIndex..location.len() {
          match location[i] {
            '&' => {
              key
            }
          }
          if location[i] == "&" {
            
        }
      }
  }
}
