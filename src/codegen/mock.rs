//! The resources of the virtualized host in fuzz and dialog modes, whose mocks the imports
//! component makes. A mock is labeled by a number the imports component gives it when it makes
//! it, counting from 1, which no other mock gets: its handle can't label it, as the handle moves
//! on with the mock, to the guest or the exports component, which frees its number, and a borrow
//! the guest passes back comes in as the mock itself, without a handle. Replay's mocks carry the
//! trace's labels.
use super::State;
use crate::Mode;
use quote::quote;
use syn::{Item, parse_quote};

impl State {
    /// Overrides wit-bindgen's `resource_from_raw_`, which the `[dtor]` of a mock calls, for its
    /// drop: replay checks the guest's with the trace, and fuzz mode records the guest's, or the
    /// host's, of a resource the guest gave it or gave back, or of a mock it made for an argument,
    /// so that the trace has the end of every mock.
    pub fn generate_mock_drop(&self) -> syn::ImplItemFn {
        let on_drop = match self.mode {
            Mode::Replay => quote! {
                if !MOVING.with(|moving| moving.get()) {
                    let name = wasm_wave::to_string(&res.to_value()).unwrap();
                    proxy::recorder::replay::check(&trace_drop(Party::Guest, Table::Imports, name));
                }
            },
            _ => quote! {
                let name = wasm_wave::to_string(&res.to_value()).unwrap();
                let line = if MOVING.with(|moving| moving.get()) {
                    Line { at: HOST_DROP_AT.get(), ..trace_drop(Party::Host, Table::Imports, name) }
                } else {
                    trace_drop(Party::Guest, Table::Imports, name)
                };
                proxy::recorder::record::record(&line);
            },
        };
        parse_quote! {
            unsafe fn resource_from_raw_(handle: *mut Option<Self>) -> Option<Self> {
                // `None` if `into_inner` moved the resource out: not a drop.
                if let Some(res) = *unsafe { Box::from_raw(handle) } {
                    #on_drop
                }
                None
            }
        }
    }
    /// Items the mocks use.
    pub fn mock_defs(&self) -> Vec<Item> {
        // The step of the host's drops, when the exports component makes them, rather than a
        // mock in answer to a call (see `host-drops`).
        let host_drop_at = match self.mode {
            Mode::Fuzz => quote! {
                static HOST_DROP_AT: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
            },
            _ => quote! {},
        };
        let file: syn::File = parse_quote! {
            thread_local! {
                // The label of the last mock.
                static MOCK_LABEL: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
                #host_drop_at
            }
            /// The label of a new mock.
            #[allow(dead_code)]
            fn next_mock_label() -> u32 {
                MOCK_LABEL.set(MOCK_LABEL.get() + 1);
                MOCK_LABEL.get()
            }
        };
        file.items
    }
}
