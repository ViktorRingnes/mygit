fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args().nth(1).ok_or("usage: openapi <path>")?;

    std::fs::write(out, server::api::document().to_pretty_json()? + "\n")?;

    Ok(())
}
