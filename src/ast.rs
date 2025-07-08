use crate::source::{Files, Source};
use crate::util::*;
use anyhow::Result;
use wit_bindgen_rust::to_rust_ident;
use wit_component::DecodedWasm;
use wit_parser::{Function, FunctionKind, Resolve, Type, WorldId, WorldItem};

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
    fn print_ty(&self, ty: &Type) -> String {
        format!("{ty:?}")
    }
    fn generate_impl<'a>(&self, funcs: impl Iterator<Item = &'a Function>, files: &mut Files) {
        let mut out = Source::default();
        out.push_str(
            r#"mod bindings;
impl Guest for Component {
"#,
        );
        for func in funcs {
            out.push_str("fn ");
            let func_name = if let FunctionKind::Constructor(_) = &func.kind {
                "new"
            } else {
                func.item_name()
            };
            out.push_str(&to_rust_ident(func_name));
            out.push_str("(");
            for (i, (name, ty)) in func.params.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("{}: {}", to_rust_ident(name), self.print_ty(ty)));
            }
            out.push_str(")");
            out.newline();
        }
        out.push_str("}\nbindings::export!(Component with_types_in bindings);");
        files.push("proxy_import/src/lib.rs", &out);
    }
    fn generate_main_wit(&self, resolve: &Resolve, id: WorldId, files: &mut Files) {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        out.push_str(
            r#"package component:proxy;
interface %record {
  %record: func(method: string, args: string, ret: string);
}
interface replay {
  replay: func(method: string, args: string, ret: string);
}
"#,
        );
        out.push_str("world imports {\n");
        out.push_str(&format!("import {};\n", ident(self.mode.to_str())));
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { id, .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import {name};\n"));
                    if !name.starts_with("wasi:") {
                        out.push_str(&format!("export {name};\n"));
                    }
                    self.generate_impl(resolve.interfaces[*id].functions.values(), files);
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        out.push_str("world exports {\n");
        out.push_str(&format!("import {};\n", ident(self.mode.to_str())));
        for (name, export) in &world.exports {
            match export {
                WorldItem::Interface { id, .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import {name};\n"));
                    out.push_str(&format!("export {name};\n"));
                }
                _ => todo!(),
            }
        }
        files.push("wit/component.wit", &out);
    }
    pub fn generate_component(&self, component: &[u8], files: &mut Files) -> Result<()> {
        let decoded = wit_component::decode(component)?;
        let (resolve, id) = match decoded {
            DecodedWasm::Component(resolve, world_id) => (resolve, world_id),
            _ => unimplemented!(),
        };
        self.generate_main_wit(&resolve, id, files);
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
