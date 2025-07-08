use std::path::Path;

mod ast;
mod source;
mod util;

fn main() -> anyhow::Result<()> {
    let path = Path::new("calculator.wasm");
    let buf = std::fs::read(path)?;
    let opts = ast::Opt::new();
    let mut files = source::Files::default();
    opts.generate_component(&buf, &mut files)?;
    for (name, content) in files.iter() {
        println!("{name}\n{content}\n");
    }
    Ok(())
}
