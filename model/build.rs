use cornucopia::{CodegenSettings, Error};
use postgres::{Client, NoTls};

// This script will generate a new cornucopia file every time your schema or queries change.
// In this example, we generate the module in our project, but
// we could also generate it elsewhere and embed the generated
// file with a `include_str` statement in your project.
fn main() -> Result<(), Error> {
    let mut client = Client::connect(
        "host=localhost user=postgres password=postgres port=5438 dbname=bevy",
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
