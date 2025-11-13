use std::error::Error;

use vergen_git2::{BuildBuilder, CargoBuilder, Emitter, Git2Builder};

fn main() -> Result<(), Box<dyn Error>> {
    Emitter::default()
        .add_instructions(&BuildBuilder::default().build_timestamp(true).build()?)?
        .add_instructions(
            &Git2Builder::default()
                .sha(false)
                .describe(true, true, Some("v[0-9]*"))
                .build()?,
        )?
        .add_instructions(&CargoBuilder::default().target_triple(true).build()?)?
        .emit()?;
    Ok(())
}
