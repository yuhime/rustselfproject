use merged_derive::{merged, MergedSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, MergedSource)]
pub struct CommonUserData {
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Serialize, Deserialize, MergedSource)]
pub struct SensitiveUserData {
    pub email: String,
    pub password: String,
}

#[merged(CommonUserData, SensitiveUserData)]
pub struct EditUserBody {
    pub session_token: String,
}

fn main() {
    let json = r#"{ "email": "johndoe@example.com", "password": "secret", "name": "john" }"#;
    let body: EditUserBody = serde_json::from_str(json).unwrap();
    println!("{:#?}", body);

    println!("{}", serde_json::to_string_pretty(&body).unwrap());
}
