use cornucopia::{CodegenSettings, Error};
use dotenv::dotenv;
use postgres::{Client, NoTls};
use std::env;

// This script will generate a new cornucopia file every time your schema or queries change.
// In this example, we generate the module in our project, but
// we could also generate it elsewhere and embed the generated
// file with a `include_str` statement in your project.
fn main() -> Result<(), Error> {
    dotenv().ok(); // Reads the .env file
    let db_port = env::var("DB_PORT").unwrap();

    if env::var("SKIP_DB_BUILD").is_ok() {
        println!("cargo:warning=Skipping cornucopia DB generation");
        return Ok(());
    }
    let mut client = Client::connect(
        &format!("host=localhost user=postgres password=postgres port={db_port} dbname=bevy")
            .to_string(),
        NoTls,
    )
    .unwrap();

    let queries_path = "queries";
    let schema_file = "../data/init.sql";
    let destination = "src/cornucopia.rs";
    let settings = CodegenSettings {
        is_async: true,
        derive_ser: false,
    };

    println!("cargo:rerun-if-changed={queries_path}");
    println!("cargo:rerun-if-changed={schema_file}");
    cornucopia::generate_live(&mut client, queries_path, Some(destination), settings)?;

    Ok(())
}
