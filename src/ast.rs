use crate::util::*;
use anyhow::Result;
use std::collections::BTreeMap;
use wit_bindgen_core::{Files, Source, wit_parser};
use wit_component::WitPrinter;
use wit_parser::*;

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
        out.push_str("export proxy:conversion/conversion;\n");
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
    pub fn generate_exports_world(&self, resolve: &Resolve, id: WorldId, files: &mut Files) {
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
        out.push_str("import proxy:conversion/conversion;\n");
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
        out = Source::default();
        if extra_imports > 0 {
            out.push_str("...imports, ");
        }
        out.push_str("... };\n");
        out.push_str("export final...;\n");
        files.push("compose.wac", out.as_bytes());
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

pub fn generate_wrapped_wits(dir: &std::path::Path) -> Result<()> {
    let mut resolve = Resolve::default();
    let (main_id, _files) = resolve.push_dir(dir)?;
    // Generate conversion interface. Not updating resolve to avoid deep cloning the packages.
    let mut resources = BTreeMap::new();
    for (_, iface) in resolve
        .interfaces
        .iter()
        .filter(|(_, iface)| iface.package.is_some_and(|id| id != main_id) && iface.name.is_some())
    {
        let pkg_id = iface.package.unwrap();
        let pkg_name = &resolve.packages[pkg_id].name;
        let iface_name = iface.name.as_ref().unwrap();
        for (ty_name, ty_id) in iface.types.iter() {
            let ty = &resolve.types[*ty_id];
            if matches!(ty.kind, TypeDefKind::Resource) {
                let mut resource =
                    format!("{}:{}/{}", pkg_name.namespace, pkg_name.name, iface_name);
                if let Some(ver) = &pkg_name.version {
                    resource.push_str(&format!("@{ver}"));
                }
                resources.insert(ty_name, resource);
            }
        }
    }
    let mut out = Source::default();
    out.push_str("package proxy:conversion;\ninterface conversion {");
    for (resource, iface) in resources.into_iter() {
        out.push_str(&format!(
            "\nuse {iface}.{{{resource} as host-{resource}}};\n",
        ));
        out.push_str(&format!(
            "use wrapped-{iface}.{{{resource} as wrapped-{resource}}};\n",
        ));
        out.push_str(&format!(
            "get-wrapped-{resource}: func(x: host-{resource}) -> wrapped-{resource};\n",
        ));
    }
    out.push_str("}\n");
    std::fs::write(dir.join("deps").join("conversion.wit"), out.as_bytes())?;
    // rename package name
    resolve.package_names = resolve
        .package_names
        .into_iter()
        .map(|(mut name, id)| {
            name.namespace = "wrapped-".to_string() + &name.namespace;
            (name, id)
        })
        .collect();
    for (_, pkg) in resolve.packages.iter_mut() {
        pkg.name.namespace = "wrapped-".to_string() + &pkg.name.namespace;
    }
    for (id, pkg) in resolve.packages.iter().filter(|(id, _)| *id != main_id) {
        let mut printer = WitPrinter::default();
        printer.print_package(&resolve, id, true)?;
        std::fs::write(
            dir.join("deps")
                .join(format!("wrapped-{}.wit", pkg.name.name)),
            printer.output.to_string(),
        )?;
    }
    Ok(())
}
