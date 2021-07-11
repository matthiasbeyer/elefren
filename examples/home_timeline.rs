#![cfg_attr(not(feature = "toml"), allow(dead_code))]
#![cfg_attr(not(feature = "toml"), allow(unused_imports))]
mod register;

use std::error;

#[cfg(feature = "toml")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let mastodon = register::get_mastodon_data()?;
    let tl = mastodon.get_home_timeline().await?;

    println!("{:#?}", tl);

    Ok(())
}

#[cfg(not(feature = "toml"))]
fn main() {
    println!(
        "examples require the `toml` feature, run this command for this example:\n\ncargo run \
         --example print_your_profile --features toml\n"
    );
}
