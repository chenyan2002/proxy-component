use crate::Mode;
use crate::instrument::InstrumentArgs;
use crate::util::*;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use wit_bindgen_core::{Files, Source, wit_parser};
use wit_component::WitPrinter;
use wit_parser::*;

pub struct Opt<'a> {
    args: &'a InstrumentArgs,
    mode: Mode,
    imports: LinkInfo,
    exports: LinkInfo,
    main: LinkInfo,
}
#[derive(Default)]
struct LinkInfo {
    imports: BTreeMap<String, LinkType>,
    exports: BTreeSet<String>,
}
enum LinkType {
    Debug,
    Recorder,
    Imports,
    Main,
    Host,
}

impl<'a> Opt<'a> {
    pub fn new(args: &'a InstrumentArgs) -> Self {
        let mode = args.mode.clone();
        Self {
            args,
            mode,
            imports: LinkInfo::default(),
            exports: LinkInfo::default(),
            main: LinkInfo::default(),
        }
    }
    fn generate_main_wit(&mut self, resolve: &Resolve, id: WorldId, files: &mut Files) {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        let recorder = "proxy:recorder/";
        out.push_str("package component:proxy;\n");
        out.push_str("world imports {\n");
        match self.mode {
            Mode::Record | Mode::Replay => {
                out.push_str(&format!(
                    "import {recorder}{}@0.1.0;\n",
                    ident(self.mode.to_str())
                ));
            }
            Mode::Fuzz => {
                out.push_str(&format!("import proxy:util/debug;\n"));
            }
            Mode::Dialog => {
                out.push_str(&format!("import proxy:util/dialog;\n"));
            }
        };
        out.push_str("export proxy:conversion/conversion;\n");
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    // Don't virtualize util imports
                    if name.starts_with("proxy:util/") {
                        self.main.imports.insert(name.clone(), LinkType::Debug);
                        out.push_str(&format!("import {name};\n"));
                        continue;
                    }
                    self.main.imports.insert(name.clone(), LinkType::Imports);
                    match self.mode {
                        Mode::Record => {
                            out.push_str(&format!("import {name};\n"));
                            out.push_str(&format!("export wrapped-{name};\n"));
                        }
                        Mode::Replay | Mode::Fuzz | Mode::Dialog => {
                            out.push_str(&format!("export {name};\n"))
                        }
                    }
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
                    self.main.exports.insert(name.clone());
                    match self.mode {
                        Mode::Record => {
                            out.push_str(&format!("import wrapped-{name};\n"));
                            out.push_str(&format!("export {name};\n"));
                        }
                        Mode::Replay | Mode::Fuzz | Mode::Dialog => {
                            out.push_str(&format!("import {name};\n"));
                        }
                    }
                }
                _ => todo!(),
            }
        }
        if matches!(self.mode, Mode::Replay | Mode::Fuzz | Mode::Dialog) {
            out.push_str("export proxy:recorder/start-replay@0.1.0;\n")
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
    }
    pub fn generate_exports_world(&self, resolve: &Resolve, id: WorldId, files: &mut Files) {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        let recorder = "proxy:recorder/";
        out.push_str("world exports {\n");
        match self.mode {
            Mode::Record | Mode::Replay => {
                out.push_str(&format!(
                    "import {recorder}{}@0.1.0;\n",
                    ident(self.mode.to_str())
                ));
            }
            Mode::Fuzz => {
                out.push_str(&format!("import proxy:util/debug;\n"));
            }
            Mode::Dialog => {
                out.push_str(&format!("import proxy:util/dialog;\n"));
            }
        };
        out.push_str("import proxy:conversion/conversion;\n");
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("import {name};\n"));
                }
                _ => todo!(),
            }
        }
        for (name, export) in &world.exports {
            match export {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    out.push_str(&format!("export {name};\n"));
                }
                _ => todo!(),
            }
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
    }
    pub fn generate_wac(
        &mut self,
        imports_wasm: &Path,
        exports_wasm: &Path,
        out_dir: &Path,
    ) -> Result<()> {
        // Get WIT from wasm to account for unused imports being optimized away
        self.load_imports(imports_wasm)?;
        self.load_exports(exports_wasm)?;
        let mut out = Source::default();
        out.push_str("package component:composed;\n");
        out.push_str("let debug = new import:debug { ... };\n");
        if !self.args.use_host_recorder {
            out.push_str("let recorder = new import:recorder { ... };\n");
        }
        out.push_str("let imports = new import:proxy {\n");
        let mut has_host = false;
        for (name, link_type) in &self.imports.imports {
            match link_type {
                LinkType::Debug => {
                    out.push_str(&format!("\"{name}\": debug[\"{name}\"] ,\n"));
                }
                LinkType::Recorder => {
                    if self.args.use_host_recorder {
                        has_host = true;
                    } else {
                        out.push_str(&format!("\"{name}\": recorder[\"{name}\"] ,\n"));
                    }
                }
                LinkType::Host => {
                    has_host = true;
                }
                LinkType::Imports | LinkType::Main => unreachable!(),
            }
        }
        if has_host {
            out.push_str("...,\n");
        }
        out.push_str("};\n");
        out.push_str("let main = new root:component {\n");
        for (name, link_type) in &self.main.imports {
            match link_type {
                LinkType::Debug => {
                    out.push_str(&format!("\"{name}\": debug[\"{name}\"] ,\n"));
                }
                LinkType::Imports => {
                    let prefix = if matches!(self.mode, Mode::Record) {
                        "wrapped-"
                    } else {
                        ""
                    };
                    out.push_str(&format!("\"{name}\": imports[\"{prefix}{name}\"] ,\n"));
                }
                LinkType::Host | LinkType::Main | LinkType::Recorder => unreachable!(),
            }
        }
        out.push_str("};\n");
        out.push_str("let final = new export:proxy {\n");
        has_host = false;
        for (name, link_type) in &self.exports.imports {
            match link_type {
                LinkType::Debug => {
                    out.push_str(&format!("\"{name}\": debug[\"{name}\"] ,\n"));
                }
                LinkType::Recorder => {
                    if self.args.use_host_recorder {
                        has_host = true;
                    } else {
                        out.push_str(&format!("\"{name}\": recorder[\"{name}\"] ,\n"));
                    }
                }
                LinkType::Host => {
                    has_host = true;
                }
                LinkType::Imports => {
                    out.push_str(&format!("\"{name}\": imports[\"{name}\"] ,\n"));
                }
                LinkType::Main => {
                    if let Some(stripped) = name.strip_prefix("wrapped-") {
                        out.push_str(&format!("\"{name}\": main[\"{stripped}\"] ,\n"));
                    } else {
                        out.push_str(&format!("\"{name}\": main[\"{name}\"] ,\n"));
                    }
                }
            }
        }
        if has_host {
            out.push_str("...,\n");
        }
        out.push_str("};\n");
        out.push_str("export final...;\n");
        std::fs::write(out_dir.join("compose.wac"), out.as_bytes())?;
        Ok(())
    }
    pub fn generate_component(
        &mut self,
        resolve: &Resolve,
        id: WorldId,
        files: &mut Files,
    ) -> Result<()> {
        self.generate_main_wit(resolve, id, files);
        files.push(
            "deps/recorder.wit",
            include_str!("../assets/recorder.wit").as_bytes(),
        );
        files.push(
            "deps/util.wit",
            include_str!("../assets/util.wit").as_bytes(),
        );
        Ok(())
    }

    pub fn generate_wrapped_wits(&self, dir: &std::path::Path) -> Result<()> {
        let mut resolve = Resolve::default();
        let (main_id, _files) = resolve.push_dir(dir)?;
        // Generate conversion interface. Not updating resolve to avoid deep cloning the packages.
        let mut resources = BTreeMap::new();
        for (_, iface) in resolve.interfaces.iter().filter(|(_, iface)| {
            iface.package.is_some_and(|id| id != main_id) && iface.name.is_some()
        }) {
            let pkg_id = iface.package.unwrap();
            let pkg_name = &resolve.packages[pkg_id].name;
            let iface_name = iface.name.as_ref().unwrap();
            for (ty_name, ty_id) in iface.types.iter() {
                let ty = &resolve.types[*ty_id];
                if matches!(ty.kind, TypeDefKind::Resource) {
                    let mut resource =
                        format!("{}:{}/{}", pkg_name.namespace, pkg_name.name, iface_name);
                    let resource_no_ver = resource.clone();
                    if let Some(ver) = &pkg_name.version {
                        resource.push_str(&format!("@{ver}"));
                    }
                    assert!(
                        resources
                            .insert(*ty_id, (ty_name, resource, resource_no_ver))
                            .is_none()
                    );
                }
            }
        }
        let mut out = Source::default();
        out.push_str("package proxy:conversion;\ninterface conversion {");
        for (resource, iface, iface_no_ver) in resources.into_values() {
            use heck::ToKebabCase;
            let func_name = format!("{iface_no_ver}-{resource}").to_kebab_case();
            match self.mode {
                Mode::Record => {
                    out.push_str(&format!(
                        "\nuse {iface}.{{{resource} as host-{func_name}}};\n",
                    ));
                    out.push_str(&format!(
                        "use wrapped-{iface}.{{{resource} as wrapped-{func_name}}};\n",
                    ));
                    out.push_str(&format!(
                        "get-wrapped-{func_name}: func(x: host-{func_name}) -> wrapped-{func_name};\n",
                    ));
                    out.push_str(&format!(
                        "get-host-{func_name}: func(x: wrapped-{func_name}) -> host-{func_name};\n",
                    ));
                }
                Mode::Replay | Mode::Fuzz | Mode::Dialog => {
                    // Add a magic separator so that codegen::generate_conversion_func can recover the resource name
                    let magic_name = format!("{iface_no_ver}-magic42-{resource}").to_kebab_case();
                    out.push_str(&format!("\nuse {iface}.{{{resource} as {func_name}}};\n"));
                    out.push_str(&format!(
                        "get-mock-{magic_name}: func(handle: u32) -> {func_name};\n"
                    ));
                }
            }
        }
        out.push_str("}\n");
        std::fs::write(dir.join("deps").join("conversion.wit"), out.as_bytes())?;
        if matches!(self.mode, Mode::Record) {
            // rename package name and generate wrapped wit
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
        }
        Ok(())
    }
    fn load_imports(&mut self, file: &Path) -> Result<()> {
        let (resolve, id) = load_wasm(file)?;
        let world = &resolve.worlds[id];
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    let link_type = match name.as_str() {
                        "proxy:util/debug" => LinkType::Debug,
                        "proxy:util/dialog" => LinkType::Host,
                        name if name.starts_with("proxy:recorder/") => LinkType::Recorder,
                        _ => LinkType::Host,
                    };
                    self.imports.imports.insert(name.clone(), link_type);
                }
                _ => todo!(),
            }
        }
        Ok(())
    }
    fn load_exports(&mut self, file: &Path) -> Result<()> {
        let (resolve, id) = load_wasm(file)?;
        let world = &resolve.worlds[id];
        for (name, import) in &world.imports {
            match import {
                WorldItem::Interface { .. } => {
                    let name = resolve.name_world_key(name);
                    let link_type = match name.as_str() {
                        "proxy:util/debug" => LinkType::Debug,
                        "proxy:conversion/conversion" => LinkType::Imports,
                        "proxy:util/dialog" => LinkType::Host,
                        name if name.starts_with("proxy:recorder/") => LinkType::Recorder,
                        name if matches!(self.mode, Mode::Record) => {
                            if let Some(stripped) = name.strip_prefix("wrapped-") {
                                if self.main.exports.contains(stripped) {
                                    LinkType::Main
                                } else {
                                    LinkType::Imports
                                }
                            } else {
                                LinkType::Host
                            }
                        }
                        name if self.main.exports.contains(name) => LinkType::Main,
                        _ => LinkType::Imports,
                    };
                    self.exports.imports.insert(name.to_string(), link_type);
                }
                _ => todo!(),
            }
        }
        Ok(())
    }
}

fn load_wasm(file: &Path) -> Result<(Resolve, WorldId)> {
    use wit_parser::decoding::{DecodedWasm, decode};
    let bytes = std::fs::read(file)?;
    let DecodedWasm::Component(resolve, id) = decode(&bytes)? else {
        panic!()
    };
    Ok((resolve, id))
}
