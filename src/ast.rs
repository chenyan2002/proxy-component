use crate::util::*;
use anyhow::Result;
use std::collections::HashMap;
use wit_bindgen_core::{Files, Source, wit_parser};
use wit_component::{Output, WitPrinter};
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
    for (_, pkg) in resolve.packages.iter().filter(|(id, _)| *id != main_id) {
        let mut printer = WitPrinter::default();
        let mut inject_resource = HashMap::new();
        printer.output.push_str("package wrapped-");
        let mut pkg_name = format!("{}:{}", pkg.name.namespace, pkg.name.name);
        if let Some(ver) = &pkg.name.version {
            pkg_name.push_str(&format!("@{ver}"));
        }
        printer.output.push_str(&pkg_name);
        printer.output.push_str(";\n");
        for (name, id) in pkg.interfaces.iter() {
            let iface = &resolve.interfaces[*id];
            for (ty_name, ty_id) in iface.types.iter() {
                let ty = &resolve.types[*ty_id];
                if matches!(ty.kind, TypeDefKind::Resource) {
                    let host_name = format!("host-{}", ty_name);
                    let mut iface_name =
                        format!("{}:{}/{}", pkg.name.namespace, pkg.name.name, name);
                    if let Some(ver) = &pkg.name.version {
                        iface_name.push_str(&format!("@{ver}"));
                    }
                    let out = format!("use {}.{{{} as {}}};", iface_name, ty_name, host_name);
                    let func =
                        format!("get-wrapped: static func(x: {}) -> {};", host_name, ty_name);
                    inject_resource.insert(ty_name, (out, func));
                }
            }
            printer.print_interface_outer(&resolve, *id, name)?;
            printer.output.indent_start();
            printer.print_interface(&resolve, *id)?;
            printer.output.indent_end();
        }
        let content = printer.output.to_string();
        let re = regex::Regex::new(r"use (\S+:)").unwrap();
        let mut content = re.replace_all(&content, "use wrapped-$1").to_string();
        for (res_name, (use_stmt, func_stmt)) in &inject_resource {
            let re_resource = regex::Regex::new(&format!(
                r"(\s*)resource\s+{}\s*(?:\{{((?:.|\n)*?)\}}|;)",
                regex::escape(res_name)
            ))
            .unwrap();
            content = re_resource
                .replace(&content, |caps: &regex::Captures| {
                    let indent = &caps[1];
                    let existing_body = caps.get(2).map_or("", |m| m.as_str());
                    let new_func = format!("{}    {}", indent, func_stmt);

                    format!(
                        "{}{}\n{}resource {} {{{}{}\n{}}}",
                        indent, use_stmt, indent, res_name, existing_body, new_func, indent
                    )
                })
                .to_string();
        }
        std::fs::write(
            dir.join("deps")
                .join(format!("wrapped-{}.wit", pkg.name.name)),
            content,
        )?;
    }
    Ok(())
}
