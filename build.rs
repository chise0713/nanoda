use std::error::Error;

use vergen_git2::{CargoBuilder, Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn Error>> {
    Emitter::default()
        .add_instructions(&Git2Builder::default().describe(true, true, None).build()?)?
        .add_instructions(&CargoBuilder::default().target_triple(true).build()?)?
        .emit()?;
    Ok(())
}
