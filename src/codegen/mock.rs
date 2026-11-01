//! The resources of the virtualized host in fuzz and dialog modes, whose mocks the imports
//! component makes: they are labeled by the handles wasmtime gives them.
use super::State;
use crate::Mode;
use quote::quote;
use syn::{Item, parse_quote};

impl State {
    /// Overrides wit-bindgen's `_resource_new`, which makes the handle of every new mock of the
    /// resource `name` of `module_path`, including those constructors return, to label the mock
    /// by its handle. As a constructor returns the mock before it has a handle, the constructor
    /// sets `PENDING_RET`, and the `ret` of the call is printed here.
    pub fn generate_mock_resource_new(
        &self,
        module_path: &[String],
        name: &str,
    ) -> syn::ImplItemFn {
        let iface = self.interfaces[module_path];
        let module = format!("[export]{}", self.resolve.id_of(iface).unwrap());
        let id = self.wit_type_id(module_path, name).unwrap();
        let link_name = format!(
            "[resource-new]{}",
            self.resolve.types[id].name.as_ref().unwrap()
        );
        let print = match self.mode {
            Mode::Fuzz => quote! { proxy::util::debug::print(&ret) },
            Mode::Dialog => quote! { proxy::util::dialog::print(0, &ret) },
            _ => unreachable!(),
        };
        parse_quote! {
            unsafe fn _resource_new(val: *mut u8) -> u32 {
                #[cfg(target_arch = "wasm32")]
                #[link(wasm_import_module = #module)]
                unsafe extern "C" {
                    #[link_name = #link_name]
                    fn new(_: *mut u8) -> i32;
                }
                #[cfg(not(target_arch = "wasm32"))]
                unsafe extern "C" fn new(_: *mut u8) -> i32 {
                    unreachable!()
                }
                let handle = unsafe { new(val) } as u32;
                let mock = unsafe { &mut *(val as *mut Option<MockedResource>) }
                    .as_mut()
                    .unwrap();
                mock.handle = handle;
                if PENDING_RET.with(|pending| pending.take()) {
                    let ret = format!("ret: {}", wasm_wave::to_string(&mock.to_value()).unwrap());
                    #print;
                }
                handle
            }
        }
    }
    /// Items the mocks use.
    pub fn mock_defs(&self) -> Vec<Item> {
        let file: syn::File = parse_quote! {
            thread_local! {
                // Set by a mocked constructor, whose `ret` `_resource_new` prints, once the mock
                // it returned has a handle.
                #[allow(dead_code)]
                static PENDING_RET: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
            }
        };
        file.items
    }
}
