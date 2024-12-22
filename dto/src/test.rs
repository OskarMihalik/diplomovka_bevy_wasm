use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Test {
    pub name: String,
    pub age: i32,
}
