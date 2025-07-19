use crate::util::*;
use anyhow::Result;
use wit_bindgen_core::{Files, Source, wit_parser};
use wit_parser::{Resolve, WorldId, WorldItem};

pub enum Mode {
    Record,
}
pub struct Opt {
    pub mode: Mode,
}

impl Opt {
    pub fn new() -> Self {
        Self { mode: Mode::Record }
    }
    fn generate_main_wit(&self, resolve: &Resolve, id: WorldId, files: &mut Files) {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        let recorder = "proxy:recorder/";
        out.push_str("package component:proxy;");
        out.push_str("world imports {\n");
        out.push_str(&format!(
            "import {recorder}{}@0.1.0;\n",
            ident(self.mode.to_str())
        ));
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import {name};\n"));
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        out.push_str("world exports {\n");
        out.push_str(&format!(
            "import {recorder}{}@0.1.0;\n",
            ident(self.mode.to_str())
        ));
        for (name, export) in &world.exports {
            match export {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import {name};\n"));
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
    }
    pub fn generate_component(
        &self,
        resolve: &Resolve,
        id: WorldId,
        files: &mut Files,
    ) -> Result<()> {
        self.generate_main_wit(resolve, id, files);
        files.push(
            "deps/recorder.wit",
            br#"
package proxy:recorder@0.1.0;
interface %record {
  %record: func(method: string, args: string, ret: string);
}
        "#,
        );
        Ok(())
    }
}

impl Mode {
    fn to_str(&self) -> &str {
        match self {
            Mode::Record => "record",
        }
    }
}
