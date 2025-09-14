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
        out.push_str("package component:proxy;\n");
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
                    out.push_str(&format!("export wrapped-{name};\n"));
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        out.push_str("world tmp-exports {\n");
        for (name, export) in &world.exports {
            match export {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import wrapped-{name};\n"));
                    out.push_str(&format!("export {name};\n"));
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
    }
    pub fn generate_exports_world(
        &self,
        resolve: &Resolve,
        id: WorldId,
        files: &mut Files,
    ) -> bool {
        let mut cnt_imports = 0;
        let mut cnt_exports = 0;
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        let recorder = "proxy:recorder/";
        out.push_str("world exports {\n");
        out.push_str(&format!(
            "import {recorder}{}@0.1.0;\n",
            ident(self.mode.to_str())
        ));
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    cnt_imports += 1;
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import {name};\n"));
                }
                _ => todo!(),
            }
        }
        for (name, export) in &world.exports {
            match export {
                WorldItem::Interface { .. } => {
                    cnt_exports += 1;
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("export {name};\n"));
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
        let extra_imports = cnt_imports - cnt_exports;
        assert!(extra_imports >= 0);
        extra_imports > 0
    }
    fn generate_wac(&self, resolve: &Resolve, id: WorldId, files: &mut Files) {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        out.push_str(
            r#"package component:composed;
let imports = new import:proxy { ... };
let main = new root:component { "#,
        );
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    let idx = name.find('/').unwrap();
                    let end = name.rfind('@').unwrap_or(name.len());
                    assert!(idx < end);
                    let name = &name[idx + 1..end];
                    out.push_str(&format!("{name}: imports.{name}, "));
                }
                _ => todo!(),
            }
        }
        out.push_str(" };\n");
        out.push_str("let final = new export:proxy { ");
        for (name, import) in &world.exports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    let idx = name.find('/').unwrap();
                    let end = name.rfind('@').unwrap_or(name.len());
                    assert!(idx < end);
                    let name = &name[idx + 1..end];
                    out.push_str(&format!("{name}: main.{name}, "));
                }
                _ => todo!(),
            }
        }
        out.push_str("... };\n");
        out.push_str("export final...;\n");
        files.push("compose.wac", out.as_bytes());
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
            include_str!("../assets/recorder.wit").as_bytes(),
        );
        self.generate_wac(resolve, id, files);
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
