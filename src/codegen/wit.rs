//! What the WIT says about the items of the bindings: their WIT names and kinds.
//!
//! wit-bindgen names each Rust item after a WIT item, so an item of the bindings is found by
//! applying wit-bindgen's naming to the WIT items, never by recovering WIT names from Rust ones.
use super::State;
use super::util::ResourceFuncKind;
use crate::wit_util::strip_wrapped;
use heck::ToUpperCamelCase;
use std::collections::BTreeMap;
use syn::Ident;
use wit_bindgen_core::{dealias, name_package_module};
use wit_bindgen_rust::to_rust_ident;
use wit_component::WitPrinter;
use wit_parser::{
    Function, FunctionKind, Handle, InterfaceId, Resolve, Type, TypeDefKind, TypeId, WorldId,
    WorldItem, WorldKey,
};

/// A WIT function of the bindings.
pub struct WitFunc {
    pub kind: Option<ResourceFuncKind>,
    /// The name traces use, e.g. `ns:pkg/iface@0.1.0.func`, `ns:pkg/iface/res@0.1.0.func` for a
    /// method or static function, or `new ns:pkg/iface/res@0.1.0` for a constructor
    pub display_name: String,
    /// The WIT name of the resource of a constructor, method or static function
    pub resource: Option<String>,
    /// The WIT types of the parameters, without a method's `self`, e.g. `list<u8>`
    pub params: Vec<String>,
    /// The WIT type of the result
    pub result: Option<String>,
}
impl WitFunc {
    /// In virtualized components, constructors return `Self`, i.e. `MockedResource`, so
    /// type-directed traits like `Dialog` or `Arbitrary` cannot recover the resource name.
    /// Returns the name to patch into the `MockedResource`, if this is a constructor.
    pub fn constructor_resource(&self) -> Option<&str> {
        match self.kind {
            Some(ResourceFuncKind::Constructor) => self.resource.as_deref(),
            _ => None,
        }
    }
}

/// A WIT type the bindings define.
pub struct WitType {
    /// e.g. `point`
    pub name: String,
    /// e.g. `ns:pkg/iface@0.1.0.point`
    pub qualified_name: String,
    /// The record's fields, the variant's or enum's cases, or the flags, in order
    pub members: Vec<String>,
    /// The WIT types of the record's fields, e.g. `option<string>`
    pub field_types: Vec<String>,
    /// For a resource, the `proxy:conversion` function that returns it
    pub conversion: Option<Ident>,
}

/// The Rust module of each interface of a world, as wit-bindgen names it (see
/// `compute_module_path` in wit-bindgen-rust).
pub fn interface_modules(resolve: &Resolve, world: WorldId) -> BTreeMap<Vec<String>, InterfaceId> {
    let world = &resolve.worlds[world];
    let imports = world.imports.iter().map(|item| (item, false));
    let exports = world.exports.iter().map(|item| (item, true));
    imports
        .chain(exports)
        .filter_map(|((key, item), is_export)| {
            let WorldItem::Interface { id, .. } = item else {
                return None;
            };
            let mut path = Vec::new();
            if is_export {
                path.push("exports".to_string());
            }
            match key {
                WorldKey::Name(name) => path.push(to_rust_ident(name)),
                WorldKey::Interface(_) => {
                    let iface = &resolve.interfaces[*id];
                    let pkg = iface.package.unwrap();
                    path.push(to_rust_ident(&resolve.packages[pkg].name.namespace));
                    path.push(name_package_module(resolve, pkg));
                    path.push(to_rust_ident(iface.name.as_ref().unwrap()));
                }
            }
            Some((path, *id))
        })
        .collect()
}

/// The `proxy:conversion` function returning each resource, by the resource's type.
pub fn conversion_funcs(resolve: &Resolve) -> BTreeMap<TypeId, String> {
    let Some((_, iface)) = resolve.interfaces.iter().find(|(_, iface)| {
        iface.name.as_deref() == Some("conversion")
            && iface.package.is_some_and(|pkg| {
                let name = &resolve.packages[pkg].name;
                name.namespace == "proxy" && name.name == "conversion"
            })
    }) else {
        return BTreeMap::new();
    };
    iface
        .functions
        .values()
        .filter_map(|func| {
            Some((
                owned_resource(resolve, func.result?)?,
                to_rust_ident(&func.name),
            ))
        })
        .collect()
}

/// The resource of `own<resource>`, through type aliases.
fn owned_resource(resolve: &Resolve, ty: Type) -> Option<TypeId> {
    let Type::Id(id) = ty else {
        return None;
    };
    match resolve.types[dealias(resolve, id)].kind {
        TypeDefKind::Handle(Handle::Own(resource)) => Some(dealias(resolve, resource)),
        _ => None,
    }
}

/// The Rust name of a WIT type (`to_upper_camel_case` in wit-bindgen-rust).
fn rust_type_name(name: &str) -> String {
    match name {
        // wit-bindgen reserves `Guest` for the traits of exported interfaces.
        "guest" => "Guest_".to_string(),
        name => name.to_upper_camel_case(),
    }
}

/// The Rust name of a WIT function, see `print_signature` in wit-bindgen-rust.
fn rust_func_name(func: &Function) -> String {
    match func.kind {
        FunctionKind::Constructor(_) => "new".to_string(),
        FunctionKind::Setter | FunctionKind::MethodSetter(_) | FunctionKind::StaticSetter(_) => {
            to_rust_ident(&format!("set-{}", func.item_name()))
        }
        _ => to_rust_ident(func.item_name()),
    }
}

impl State {
    /// An interface, or one of its resources, named as in traces, with the version last, as
    /// component-ui names them: e.g. `ns:pkg/iface@0.1.0`, or `ns:pkg/iface/res@0.1.0`. In record
    /// mode, the guest sees each wrapped interface under its own name.
    fn interface_name(&self, id: InterfaceId, resource: Option<&str>) -> String {
        let iface = &self.resolve.interfaces[id];
        let pkg = &self.resolve.packages[iface.package.unwrap()].name;
        let namespace = strip_wrapped(&pkg.namespace).unwrap_or(&pkg.namespace);
        let mut name = format!("{namespace}:{}/{}", pkg.name, iface.name.as_ref().unwrap());
        if let Some(resource) = resource {
            name.push_str(&format!("/{resource}"));
        }
        if let Some(version) = &pkg.version {
            name.push_str(&format!("@{version}"));
        }
        name
    }
    /// A WIT type as WIT source writes it, e.g. `list<u8>` or `borrow<x>`.
    fn type_name(&self, ty: &Type) -> String {
        let mut printer = WitPrinter::default();
        printer.print_type_name(&self.resolve, ty).unwrap();
        printer.output.to_string()
    }
    /// The WIT type of a Rust type defined in `module_path`.
    pub fn wit_type_id(&self, module_path: &[String], name: &str) -> Option<TypeId> {
        let iface = &self.resolve.interfaces[*self.interfaces.get(module_path)?];
        iface
            .types
            .iter()
            .find(|(wit_name, _)| rust_type_name(wit_name) == name)
            .map(|(_, id)| *id)
    }
    pub fn wit_type(&self, module_path: &[String], id: TypeId) -> WitType {
        let ty = &self.resolve.types[id];
        let name = ty.name.clone().unwrap();
        let field_types = match &ty.kind {
            TypeDefKind::Record(record) => record
                .fields
                .iter()
                .map(|f| self.type_name(&f.ty))
                .collect(),
            _ => Vec::new(),
        };
        let members = match &ty.kind {
            TypeDefKind::Record(record) => record.fields.iter().map(|f| f.name.clone()).collect(),
            TypeDefKind::Variant(variant) => variant.cases.iter().map(|c| c.name.clone()).collect(),
            TypeDefKind::Enum(enum_) => enum_.cases.iter().map(|c| c.name.clone()).collect(),
            TypeDefKind::Flags(flags) => flags.flags.iter().map(|f| f.name.clone()).collect(),
            _ => Vec::new(),
        };
        let iface = self.interfaces[module_path];
        WitType {
            qualified_name: format!("{}.{name}", self.interface_name(iface, None)),
            name,
            members,
            field_types,
            conversion: self
                .conversions
                .get(&id)
                .map(|func| syn::parse_str(func).unwrap()),
        }
    }
    /// The `proxy:conversion` function returning the resource `name` of `module_path`.
    pub fn conversion_func(&self, module_path: &[String], name: &str) -> Ident {
        let id = self.wit_type_id(module_path, name);
        let func = id.and_then(|id| self.conversions.get(&id));
        let func = func.unwrap_or_else(|| {
            panic!(
                "no conversion function for {}::{name}",
                module_path.join("::")
            )
        });
        syn::parse_str(func).unwrap()
    }
    /// The WIT name of the resource a `proxy:conversion` function returns.
    pub fn conversion_resource(&self, func: &Ident) -> String {
        let (id, _) = self
            .conversions
            .iter()
            .find(|(_, name)| *func == name)
            .unwrap_or_else(|| panic!("{func} is not a conversion function"));
        self.resolve.types[*id].name.clone().unwrap()
    }
    /// The WIT function of a Rust function of `module_path`, which belongs to the Rust
    /// `resource`, if any. `None` for Rust functions without one.
    pub fn wit_func(
        &self,
        module_path: &[String],
        resource: Option<&str>,
        func: &Ident,
    ) -> Option<WitFunc> {
        let iface_id = *self.interfaces.get(module_path)?;
        let iface = &self.resolve.interfaces[iface_id];
        let resource_name = |func: &Function| {
            let id = func.kind.resource()?;
            self.resolve.types[id].name.clone()
        };
        let wit_func = iface.functions.values().find(|f| {
            let rust_resource = resource_name(f).map(|name| rust_type_name(&name));
            rust_resource.as_deref() == resource && *func == rust_func_name(f)
        })?;
        let kind = match wit_func.kind {
            FunctionKind::Constructor(_) => Some(ResourceFuncKind::Constructor),
            FunctionKind::Method(_)
            | FunctionKind::AsyncMethod(_)
            | FunctionKind::MethodGetter(_)
            | FunctionKind::MethodSetter(_) => Some(ResourceFuncKind::Method),
            _ => None,
        };
        let resource = resource_name(wit_func);
        // As component-ui names calls: a method's `self` is its first argument, so methods and
        // static functions are named alike.
        let callee = self.interface_name(iface_id, resource.as_deref());
        let display_name = match kind {
            Some(ResourceFuncKind::Constructor) => format!("new {callee}"),
            _ => format!("{callee}.{}", wit_func.item_name()),
        };
        let self_params = matches!(kind, Some(ResourceFuncKind::Method)) as usize;
        Some(WitFunc {
            kind,
            display_name,
            resource,
            params: wit_func.params[self_params..]
                .iter()
                .map(|param| self.type_name(&param.ty))
                .collect(),
            result: wit_func.result.as_ref().map(|ty| self.type_name(ty)),
        })
    }
    /// Like `wit_func`, for functions the bindings generate from WIT.
    pub fn expect_wit_func(
        &self,
        module_path: &[String],
        resource: Option<&str>,
        func: &Ident,
    ) -> WitFunc {
        self.wit_func(module_path, resource, func)
            .unwrap_or_else(|| {
                panic!(
                    "no WIT function for {}::{}{func}",
                    module_path.join("::"),
                    resource.map(|r| format!("{r}::")).unwrap_or_default()
                )
            })
    }
}
