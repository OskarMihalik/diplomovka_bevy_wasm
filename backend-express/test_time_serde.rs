use time::macros::datetime;
fn main() {
    let dt = datetime!(2026-05-14 09:22:32.105);
    println!("{}", serde_json::to_string(&dt).unwrap());
}
