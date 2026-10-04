use crate::Mode;
use crate::instrument::InstrumentArgs;
use crate::util::*;
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use wit_bindgen_core::{Files, Source};
use wit_component::WitPrinter;
use wit_parser::*;

pub struct Opt<'a> {
    args: &'a InstrumentArgs,
    mode: Mode,
    /// How each import of the main component is linked.
    main_imports: BTreeMap<String, LinkType>,
    main_exports: BTreeSet<String>,
    /// How each import of the generated `imports` component is linked.
    imports_links: BTreeMap<String, LinkType>,
    /// How each import of the generated `exports` component is linked.
    exports_links: BTreeMap<String, LinkType>,
    need_debug: bool,
}
/// The instance in the composed component that provides an import.
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
            main_imports: BTreeMap::new(),
            main_exports: BTreeSet::new(),
            imports_links: BTreeMap::new(),
            exports_links: BTreeMap::new(),
            need_debug: false,
        }
    }
    fn push_mode_imports(&self, out: &mut Source) {
        match self.mode {
            Mode::Record | Mode::Replay => {
                out.push_str(&format!(
                    "import proxy:recorder/{}@0.1.0;\n",
                    ident(self.mode.to_str())
                ));
            }
            Mode::Fuzz => out.push_str("import proxy:util/debug;\n"),
            Mode::Dialog => out.push_str("import proxy:util/dialog;\n"),
        }
    }
    fn generate_main_wit(
        &mut self,
        resolve: &Resolve,
        id: WorldId,
        files: &mut Files,
    ) -> Result<()> {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        out.push_str("package component:proxy;\n");
        out.push_str("world imports {\n");
        self.push_mode_imports(&mut out);
        out.push_str("export proxy:conversion/conversion;\n");
        for name in interface_names(resolve, &world.imports)? {
            // Don't virtualize util imports
            if name.starts_with("proxy:util/") {
                self.need_debug = true;
                out.push_str(&format!("import {name};\n"));
                self.main_imports.insert(name, LinkType::Debug);
                continue;
            }
            match self.mode {
                Mode::Record => {
                    out.push_str(&format!("import {name};\n"));
                    out.push_str(&format!("export {};\n", wrapped_name(&name)));
                }
                Mode::Replay | Mode::Fuzz | Mode::Dialog => {
                    out.push_str(&format!("export {name};\n"))
                }
            }
            self.main_imports.insert(name, LinkType::Imports);
        }
        out.push_str("}\n");
        out.push_str("world tmp-exports {\n");
        for name in interface_names(resolve, &world.exports)? {
            match self.mode {
                Mode::Record => {
                    out.push_str(&format!("import {};\n", wrapped_name(&name)));
                    out.push_str(&format!("export {name};\n"));
                }
                Mode::Replay | Mode::Fuzz | Mode::Dialog => {
                    out.push_str(&format!("import {name};\n"));
                }
            }
            self.main_exports.insert(name);
        }
        if matches!(self.mode, Mode::Replay | Mode::Fuzz | Mode::Dialog) {
            out.push_str("export proxy:recorder/start-replay@0.1.0;\n")
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
        Ok(())
    }
    pub fn generate_exports_world(
        &self,
        resolve: &Resolve,
        id: WorldId,
        files: &mut Files,
    ) -> Result<()> {
        let mut out = Source::default();
        let world = &resolve.worlds[id];
        out.push_str("world exports {\n");
        self.push_mode_imports(&mut out);
        out.push_str("import proxy:conversion/conversion;\n");
        for name in interface_names(resolve, &world.imports)? {
            out.push_str(&format!("import {name};\n"));
        }
        for name in interface_names(resolve, &world.exports)? {
            out.push_str(&format!("export {name};\n"));
        }
        out.push_str("}\n");
        files.push("component.wit", out.as_bytes());
        Ok(())
    }
    pub fn generate_wac(
        &mut self,
        imports_wasm: &Path,
        exports_wasm: &Path,
        out_dir: &Path,
    ) -> Result<()> {
        self.load_imports(imports_wasm)?;
        self.load_exports(exports_wasm)?;
        let mut out = Source::default();
        out.push_str("package component:composed;\n");
        if self.need_debug {
            out.push_str("let debug = new import:debug { ... };\n");
        }
        if !self.args.use_host_recorder {
            out.push_str("let recorder = new import:recorder { ... };\n");
        }
        out.push_str("let imports = new import:proxy {\n");
        let mut has_host = false;
        for (name, link_type) in &self.imports_links {
            has_host |= self.push_external_link(&mut out, name, link_type);
        }
        close_instance(&mut out, has_host);
        out.push_str("let main = new root:component {\n");
        for (name, link_type) in &self.main_imports {
            match link_type {
                LinkType::Debug => push_link(&mut out, name, "debug", name),
                LinkType::Imports => {
                    let export = match self.mode {
                        Mode::Record => wrapped_name(name),
                        _ => name.clone(),
                    };
                    push_link(&mut out, name, "imports", &export);
                }
                LinkType::Host | LinkType::Main | LinkType::Recorder => unreachable!(),
            }
        }
        out.push_str("};\n");
        out.push_str("let final = new export:proxy {\n");
        has_host = false;
        for (name, link_type) in &self.exports_links {
            match link_type {
                LinkType::Imports => push_link(&mut out, name, "imports", name),
                LinkType::Main => {
                    push_link(&mut out, name, "main", strip_wrapped(name).unwrap_or(name))
                }
                _ => has_host |= self.push_external_link(&mut out, name, link_type),
            }
        }
        close_instance(&mut out, has_host);
        out.push_str("export final...;\n");
        std::fs::write(out_dir.join("compose.wac"), out.as_bytes())?;
        Ok(())
    }
    /// Links an import that is provided outside of the generated and main components.
    /// Returns true if the import is left for the host to provide.
    fn push_external_link(&self, out: &mut Source, name: &str, link_type: &LinkType) -> bool {
        match link_type {
            LinkType::Debug => push_link(out, name, "debug", name),
            LinkType::Recorder if !self.args.use_host_recorder => {
                push_link(out, name, "recorder", name)
            }
            LinkType::Recorder | LinkType::Host => return true,
            LinkType::Imports | LinkType::Main => unreachable!(),
        }
        false
    }
    pub fn generate_component(
        &mut self,
        resolve: &Resolve,
        id: WorldId,
        files: &mut Files,
    ) -> Result<()> {
        let main_pkg = resolve.worlds[id].package.unwrap();
        self.generate_main_wit(resolve, id, files)?;
        self.generate_conversion_wit(resolve, main_pkg, files);
        if matches!(self.mode, Mode::Record) {
            generate_wrapped_wits(resolve, main_pkg, files)?;
        }
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
    /// Generate the `proxy:conversion` interface for converting resources from
    /// non-main packages between the generated components.
    fn generate_conversion_wit(&self, resolve: &Resolve, main_pkg: PackageId, files: &mut Files) {
        let has_version = package_with_version(resolve);
        let mut resources = BTreeMap::new();
        for (_, iface) in resolve.interfaces.iter().filter(|(_, iface)| {
            iface.package.is_some_and(|id| id != main_pkg) && iface.name.is_some()
        }) {
            let pkg_id = iface.package.unwrap();
            let pkg_name = &resolve.packages[pkg_id].name;
            let iface_name = iface.name.as_ref().unwrap();
            for (ty_name, ty_id) in iface.types.iter() {
                let ty = &resolve.types[*ty_id];
                if matches!(ty.kind, TypeDefKind::Resource) {
                    let mut resource =
                        format!("{}:{}/{}", pkg_name.namespace, pkg_name.name, iface_name);
                    let mut bindgen_name = format!("{}:{}", pkg_name.namespace, pkg_name.name);
                    if let Some(ver) = &pkg_name.version {
                        resource.push_str(&format!("@{ver}"));
                        if has_version.contains(&pkg_id) {
                            bindgen_name.push_str(&format!("{ver}"));
                        }
                    }
                    bindgen_name.push_str(&format!("/{}", iface_name));
                    assert!(
                        resources
                            .insert(*ty_id, (ty_name, resource, bindgen_name))
                            .is_none()
                    );
                }
            }
        }
        let mut out = Source::default();
        out.push_str("package proxy:conversion;\ninterface conversion {");
        for (resource, iface, bindgen_name) in resources.into_values() {
            use heck::ToKebabCase;
            let func_name = format!("{bindgen_name}-{resource}").to_kebab_case();
            match self.mode {
                Mode::Record => {
                    let wrapped_iface = wrapped_name(&iface);
                    let wrapped_func = wrapped_name(&func_name);
                    out.push_str(&format!(
                        "\nuse {iface}.{{{resource} as host-{func_name}}};\n",
                    ));
                    out.push_str(&format!(
                        "use {wrapped_iface}.{{{resource} as {wrapped_func}}};\n",
                    ));
                    out.push_str(&format!(
                        "get-{wrapped_func}: func(x: host-{func_name}) -> {wrapped_func};\n",
                    ));
                    out.push_str(&format!(
                        "get-host-{func_name}: func(x: {wrapped_func}) -> host-{func_name};\n",
                    ));
                }
                Mode::Replay | Mode::Fuzz | Mode::Dialog => {
                    out.push_str(&format!("\nuse {iface}.{{{resource} as {func_name}}};\n"));
                    out.push_str(&format!(
                        "get-mock-{func_name}: func(handle: u32) -> {func_name};\n"
                    ));
                }
            }
        }
        out.push_str("}\n");
        files.push("deps/conversion.wit", out.as_bytes());
    }
    fn load_imports(&mut self, file: &Path) -> Result<()> {
        for name in import_names(file)? {
            let link_type = match name.as_str() {
                "proxy:util/debug" => {
                    self.need_debug = true;
                    LinkType::Debug
                }
                "proxy:util/dialog" => LinkType::Host,
                name if name.starts_with("proxy:recorder/") => LinkType::Recorder,
                _ => LinkType::Host,
            };
            self.imports_links.insert(name, link_type);
        }
        Ok(())
    }
    fn load_exports(&mut self, file: &Path) -> Result<()> {
        for name in import_names(file)? {
            let link_type = match name.as_str() {
                "proxy:util/debug" => {
                    self.need_debug = true;
                    LinkType::Debug
                }
                "proxy:conversion/conversion" => LinkType::Imports,
                "proxy:util/dialog" => LinkType::Host,
                name if name.starts_with("proxy:recorder/") => LinkType::Recorder,
                name if matches!(self.mode, Mode::Record) => match strip_wrapped(name) {
                    Some(stripped) if self.main_exports.contains(stripped) => LinkType::Main,
                    Some(_) => LinkType::Imports,
                    None => LinkType::Host,
                },
                name if self.main_exports.contains(name) => LinkType::Main,
                _ => LinkType::Imports,
            };
            self.exports_links.insert(name, link_type);
        }
        Ok(())
    }
}

/// Generate a copy of every non-main package with the namespace prefixed by `wrapped-`.
fn generate_wrapped_wits(resolve: &Resolve, main_pkg: PackageId, files: &mut Files) -> Result<()> {
    let mut resolve = resolve.clone();
    resolve.package_names = resolve
        .package_names
        .into_iter()
        .map(|(mut name, id)| {
            name.namespace = wrapped_name(&name.namespace);
            (name, id)
        })
        .collect();
    for (_, pkg) in resolve.packages.iter_mut() {
        pkg.name.namespace = wrapped_name(&pkg.name.namespace);
    }
    for (id, pkg) in resolve.packages.iter().filter(|(id, _)| *id != main_pkg) {
        let mut printer = WitPrinter::default();
        printer.print_package(&resolve, id, true)?;
        let filename = match &pkg.name.version {
            Some(ver) => format!("{}@{}.wit", pkg.name.name, ver),
            None => format!("{}.wit", pkg.name.name),
        };
        files.push(
            &format!("deps/{}", wrapped_name(&filename)),
            printer.output.to_string().as_bytes(),
        );
    }
    Ok(())
}

fn push_link(out: &mut Source, name: &str, instance: &str, export: &str) {
    out.push_str(&format!("\"{name}\": {instance}[\"{export}\"] ,\n"));
}
fn close_instance(out: &mut Source, has_host: bool) {
    if has_host {
        out.push_str("...,\n");
    }
    out.push_str("};\n");
}

/// Names of the interfaces in a world's imports or exports.
fn interface_names<'r>(
    resolve: &Resolve,
    items: impl IntoIterator<Item = (&'r WorldKey, &'r WorldItem)>,
) -> Result<Vec<String>> {
    items
        .into_iter()
        .map(|(key, item)| match item {
            WorldItem::Interface { .. } => Ok(resolve.name_world_key(key)),
            _ => bail!(
                "unsupported world item `{}`: only interfaces are supported",
                resolve.name_world_key(key)
            ),
        })
        .collect()
}

/// Interface imports of a built component. Reading them from the wasm
/// accounts for unused imports being optimized away.
fn import_names(file: &Path) -> Result<Vec<String>> {
    use wit_parser::decoding::{DecodedWasm, decode};
    let bytes = std::fs::read(file)?;
    let DecodedWasm::Component(resolve, id) = decode(&bytes)? else {
        bail!("{} is not a component", file.display());
    };
    interface_names(&resolve, &resolve.worlds[id].imports)
}
