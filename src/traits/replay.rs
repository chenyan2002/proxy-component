//! `Replay`, what replay mode does with the values of the main component's interfaces, as `Fuzz`
//! is for fuzz mode: `replay` makes a value the host gives the guest from the trace's WAVE text,
//! and `consume` takes one the guest gives it. A local trait, rather than wasm-wave's `ToRust`, so
//! that it can be implemented for wit-bindgen's streams, futures and error-contexts.
use crate::codegen::util::make_path;
use crate::codegen::wit::WitType;
use crate::traits::{Trait, host_end_defs, new_mock};
use quote::quote;
use syn::{Item, ItemEnum, ItemStruct, parse_quote};

pub struct ReplayTrait {
    /// Whether the bindings have streams, whose `Replay` impl needs their `wit_stream` module
    pub has_streams: bool,
    /// Whether the bindings have futures, whose `Replay` impl needs their `wit_future` module
    pub has_futures: bool,
}

impl Trait for ReplayTrait {
    fn resource_trait(
        &self,
        module_path: &[String],
        resource: &ItemStruct,
        wit: &WitType,
    ) -> Vec<Item> {
        let resource_path = make_path(module_path, &resource.ident.to_string());
        let wit_name = &wit.name;
        if module_path[0] != "exports" {
            // A mock the imports component makes, with the trace's label, or a resource of the
            // guest, which the host keeps, until the trace drops it.
            let mock = new_mock(wit, quote! { handle });
            let consume = if wit.conversion.is_none() {
                quote! {
                    fn consume(self) {
                        std::mem::forget(self);
                    }
                }
            } else {
                quote! {}
            };
            vec![
                parse_quote! {
                impl Replay for #resource_path {
                    fn replay(value: &Value) -> Self {
                        let label = value.unwrap_handle();
                        let handle = label
                            .strip_prefix(&format!("{}-", #wit_name))
                            .and_then(|s| s.parse::<u32>().ok())
                            .expect("invalid handle label");
                        #mock
                    }
                    #consume
                }
                },
                parse_quote! {
                impl<'a> Replay for &'a #resource_path {
                    fn replay(value: &Value) -> Self {
                        let label = value.unwrap_handle();
                        let handle = label
                            .strip_prefix(&format!("borrow-{}-", #wit_name))
                            .and_then(|s| s.parse::<u32>().ok())
                            .expect("invalid handle label");
                        SCOPED_ALLOC.with(|alloc| {
                            let mut alloc = alloc.borrow_mut();
                            alloc.alloc(#mock)
                        })
                    }
                }
                },
            ]
        } else {
            // The imports component's own mock of a host resource, labeled as the trace has it.
            let borrow_path = make_path(module_path, &format!("{}Borrow<'a>", resource.ident));
            vec![
                parse_quote! {
                impl Replay for #resource_path {
                    fn replay(value: &Value) -> Self {
                        let label = value.unwrap_handle();
                        let handle = label
                            .strip_prefix(&format!("{}-", #wit_name))
                            .and_then(|s| s.parse::<u32>().ok())
                            .expect("invalid handle label");
                        #resource_path::new(MockedResource {
                            handle,
                            name: #wit_name.to_string(),
                        })
                    }
                }
                },
                parse_quote! {
                impl<'a> Replay for #borrow_path {
                    fn replay(_value: &Value) -> Self {
                        unreachable!()
                    }
                }
                },
            ]
        }
    }
    fn struct_trait(
        &self,
        module_path: &[String],
        struct_item: &ItemStruct,
        wit: &WitType,
    ) -> Vec<Item> {
        let struct_name = make_path(module_path, &struct_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = struct_item.generics.split_for_impl();
        let wit_names = &wit.members;
        let field_names: Vec<_> = match &struct_item.fields {
            syn::Fields::Unit => Vec::new(),
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| f.ident.clone().unwrap())
                .collect(),
            syn::Fields::Unnamed(_) => unreachable!(),
        };
        vec![parse_quote! {
        impl #impl_generics Replay for #struct_name #ty_generics #where_clause {
            #[allow(unused_variables)]
            fn replay(value: &Value) -> Self {
                let fields: std::collections::BTreeMap<_, _> = value.unwrap_record().collect();
                #struct_name {
                    #(#field_names: Replay::replay(&fields[#wit_names])),*
                }
            }
            #[allow(unused_variables)]
            fn consume(self) {
                #(Replay::consume(self.#field_names);)*
            }
        }
        }]
    }
    fn enum_trait(&self, module_path: &[String], enum_item: &ItemEnum, wit: &WitType) -> Vec<Item> {
        let enum_name = make_path(module_path, &enum_item.ident.to_string());
        let (impl_generics, ty_generics, where_clause) = enum_item.generics.split_for_impl();
        let arms = enum_item
            .variants
            .iter()
            .zip(&wit.members)
            .map(|(variant, wit_name)| {
                let tag = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unit => quote! {
                        (ref case, None) if case == #wit_name => #enum_name::#tag
                    },
                    syn::Fields::Unnamed(_) => quote! {
                        (ref case, Some(val)) if case == #wit_name => #enum_name::#tag(Replay::replay(&val))
                    },
                    syn::Fields::Named(_) => unreachable!(),
                }
            });
        let consume_arms = enum_item.variants.iter().map(|variant| {
            let tag = &variant.ident;
            match &variant.fields {
                syn::Fields::Unit => quote! { #enum_name::#tag => {} },
                syn::Fields::Unnamed(_) => quote! { #enum_name::#tag(x) => Replay::consume(x) },
                syn::Fields::Named(_) => unreachable!(),
            }
        });
        vec![parse_quote! {
        impl #impl_generics Replay for #enum_name #ty_generics #where_clause {
            fn replay(value: &Value) -> Self {
                match value.unwrap_variant() {
                    #(#arms),*,
                    _ => unreachable!(),
                }
            }
            fn consume(self) {
                match self {
                    #(#consume_arms),*
                }
            }
        }
        }]
    }
    fn flag_trait(
        &self,
        module_path: &[String],
        item: &crate::codegen::ItemFlag,
        wit: &WitType,
    ) -> Vec<Item> {
        let flag_path = make_path(module_path, &item.name.to_string());
        let set_flags = item.flags.iter().zip(&wit.members).map(|(f, wit_name)| {
            quote! {
                if flags.iter().any(|flag| flag == #wit_name) {
                    res |= #flag_path::#f;
                }
            }
        });
        vec![parse_quote! {
        impl Replay for #flag_path {
            fn replay(value: &Value) -> Self {
                let flags: Vec<_> = value.unwrap_flags().map(|f| f.into_owned()).collect();
                let mut res = #flag_path::empty();
                #(#set_flags)*
                res
            }
        }
        }]
    }
    fn trait_defs(&self) -> Vec<Item> {
        let ast: syn::File = parse_quote! {
            trait Replay: Sized {
                // A value the host gives the guest, from its WAVE text in the trace.
                fn replay(value: &Value) -> Self;
                // From the payload of a result's or a variant's case, which `()` has none of.
                fn replay_payload(value: Option<&Value>) -> Self {
                    Self::replay(value.expect("the case has a payload"))
                }
                // What the host does with a value the guest gives it: by default, drop it.
                // Streams and futures override it, which the trace reads, and the guest's
                // resources, which the host keeps until the trace drops them; records, variants
                // and the like pass it on to their parts.
                #[allow(dead_code)]
                fn consume(self) {}
            }
            /// The WAVE value of `text`, of type `T`.
            #[allow(dead_code)]
            fn wave_value<T: ValueTyped>(text: &str) -> Value {
                wasm_wave::from_str(&T::value_type(), text).unwrap()
            }
            macro_rules! impl_replay_primitive {
                ($($t:ty),*) => {
                    $(
                        impl Replay for $t {
                            fn replay(value: &Value) -> Self {
                                value.to_rust()
                            }
                        }
                    )*
                };
            }
            impl_replay_primitive!(bool, u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, char, String);
            impl Replay for () {
                fn replay(_value: &Value) -> Self {}
                fn replay_payload(_value: Option<&Value>) -> Self {}
            }
            impl<T: Replay> Replay for Vec<T> {
                fn replay(value: &Value) -> Self {
                    value.unwrap_list().map(|x| T::replay(&x)).collect()
                }
                fn consume(self) {
                    self.into_iter().for_each(Replay::consume);
                }
            }
            impl<T: Replay> Replay for Option<T> {
                fn replay(value: &Value) -> Self {
                    value.unwrap_option().map(|x| T::replay(&x))
                }
                fn consume(self) {
                    self.into_iter().for_each(Replay::consume);
                }
            }
            impl<T: Replay, E: Replay> Replay for core::result::Result<T, E> {
                fn replay(value: &Value) -> Self {
                    match value.unwrap_result() {
                        Ok(x) => Ok(T::replay_payload(x.as_deref())),
                        Err(x) => Err(E::replay_payload(x.as_deref())),
                    }
                }
                fn consume(self) {
                    match self {
                        Ok(x) => x.consume(),
                        Err(x) => x.consume(),
                    }
                }
            }
            macro_rules! impl_replay_tuple {
                ($($T:ident),+) => {
                    impl<$($T: Replay),+> Replay for ($($T,)+) {
                        fn replay(value: &Value) -> Self {
                            let mut items = value.unwrap_tuple();
                            ($($T::replay(&items.next().unwrap()),)+)
                        }
                        #[allow(non_snake_case)]
                        fn consume(self) {
                            let ($($T,)+) = self;
                            $($T.consume();)+
                        }
                    }
                };
            }
            impl_replay_tuple!(T1);
            impl_replay_tuple!(T1, T2);
            impl_replay_tuple!(T1, T2, T3);
            impl_replay_tuple!(T1, T2, T3, T4);
            impl_replay_tuple!(T1, T2, T3, T4, T5);
            impl_replay_tuple!(T1, T2, T3, T4, T5, T6);
            impl_replay_tuple!(T1, T2, T3, T4, T5, T6, T7);
            impl_replay_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
            impl_replay_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
            // A constructor's.
            impl Replay for MockedResource {
                fn replay(value: &Value) -> Self {
                    let label = value.unwrap_handle();
                    let (name, handle) = label
                        .rsplit_once('-')
                        .and_then(|(name, handle_str)| {
                            handle_str.parse::<u32>().ok().map(|h| (name.to_string(), h))
                        })
                        .expect("invalid handle label");
                    MockedResource { handle, name }
                }
            }
            // The trace doesn't have an error-context's message, only its label.
            impl Replay for wit_bindgen::rt::async_support::ErrorContext {
                fn replay(_value: &Value) -> Self {
                    Self::new("")
                }
            }
        };
        let mut items = ast.items;
        items.extend(self.async_defs());
        items
    }
}

impl ReplayTrait {
    /// `Replay` for streams and futures. The host's ends of those it gives the guest, which it
    /// makes, and of those the guest gives it, are jobs, which take the trace's host lines about
    /// the stream or future, and perform each in its step, and check what the guest wrote. A
    /// stream or future the host makes is paired with the trace's label, which its writer's
    /// stands for (see `crates/trace`'s renaming).
    fn async_defs(&self) -> Vec<Item> {
        let mut items = host_end_defs(self.has_streams, self.has_futures);
        if self.has_streams || self.has_futures {
            let ast: syn::File = parse_quote! {
                /// The trace's label of a stream or future, as the value that hands it out has it.
                fn trace_label(value: &Value) -> Label {
                    Label { table: TRACE_TABLE, name: format!("#{}", value.unwrap_handle()) }
                }
                /// The label of a stream or future, by the end this component keeps.
                fn label_of(end: &impl ToValue) -> Label {
                    Label { table: TRACE_TABLE, name: handle_label(end) }
                }
                fn guest_line(event: Event) -> Line {
                    Line { by: Party::Guest, at: None, event }
                }
                /// The next host line about `label`, which the job performs in its step.
                async fn next_line(label: &Label) -> Option<Line> {
                    let line = proxy::recorder::replay::take_host_line(&LineKey::Handle(label.clone()))?;
                    trace_reach(line.at).await;
                    Some(line)
                }
            };
            items.extend(ast.items);
        }
        if self.has_streams {
            let ast: syn::File = parse_quote! {
                impl<T> Replay for wit_bindgen::StreamReader<T>
                where
                    T: Replay + ToValue + ValueTyped + wit_stream::StreamPayload,
                {
                    // Writes the trace's chunks, then ends the stream, as the trace has it.
                    fn replay(value: &Value) -> Self {
                        let (mut writer, reader, name) = host_stream::<T>(false);
                        let label = Label { table: TRACE_TABLE, name };
                        proxy::recorder::replay::pair(&trace_label(value), &label);
                        spawn(async move {
                            loop {
                                let Some(line) = next_line(&label).await else {
                                    // The guest dropped the stream, which the host saw at its next
                                    // write; the guest doesn't read that, so the host just stops.
                                    drop(writer);
                                    proxy::recorder::replay::check(&guest_line(Event::Drop(label)));
                                    return;
                                };
                                match line.event {
                                    Event::Write(chunk) => {
                                        let items: Vec<T> = Replay::replay(&wave_value::<Vec<T>>(&chunk.items));
                                        let rest = writer.write_all(items).await;
                                        assert!(rest.is_empty(), "the guest dropped {label:?} before it read {}", chunk.items);
                                    }
                                    Event::Drop(_) => return drop(writer),
                                    event => unreachable!("the host can't do {event:?} with its stream"),
                                }
                            }
                        });
                        reader
                    }
                    // Reads as the trace has it, and checks what the guest wrote.
                    fn consume(mut self) {
                        let label = label_of(&self);
                        spawn(async move {
                            loop {
                                let Some(line) = next_line(&label).await else {
                                    // The guest ended the stream, which the host's next read sees.
                                    let (result, items) = self.read(Vec::with_capacity(1)).await;
                                    assert!(
                                        matches!(result, wit_bindgen::StreamResult::Dropped) && items.is_empty(),
                                        "the guest wrote to {label:?} after the trace's last read",
                                    );
                                    proxy::recorder::replay::check(&guest_line(Event::Drop(label)));
                                    return;
                                };
                                match line.event {
                                    // As many items as the trace has, which the guest's writes
                                    // split alike.
                                    Event::Read(chunk) => {
                                        let count = wave_value::<Vec<T>>(&chunk.items).unwrap_list().count();
                                        let (_, items) = self.read(Vec::with_capacity(count)).await;
                                        let wave = wasm_wave::to_string(&items.to_value()).unwrap();
                                        assert_eq!(wave, chunk.items, "the guest wrote other items to {label:?}");
                                        host_dropping(move || drop(items));
                                    }
                                    Event::Drop(_) => return drop(self),
                                    event => unreachable!("the host can't do {event:?} with the guest's stream"),
                                }
                            }
                        });
                    }
                }
            };
            items.extend(ast.items);
        }
        if self.has_futures {
            let ast: syn::File = parse_quote! {
                impl<T> Replay for wit_bindgen::FutureReader<T>
                where
                    T: Replay + ToValue + ValueTyped + wit_future::FuturePayload,
                {
                    // Writes the trace's value, unless the guest dropped the future first.
                    fn replay(value: &Value) -> Self {
                        let (writer, reader, name) = host_future::<T>(false);
                        let label = Label { table: TRACE_TABLE, name };
                        proxy::recorder::replay::pair(&trace_label(value), &label);
                        spawn(async move {
                            let Some(line) = next_line(&label).await else {
                                // The trace has no value to write; the guest dropped its end, so
                                // nothing sees the writer that is kept unwritten.
                                std::mem::forget(writer);
                                proxy::recorder::replay::check(&guest_line(Event::Drop(label)));
                                return;
                            };
                            let Event::Value(value) = line.event else {
                                unreachable!("the host can't do {:?} with its future", line.event);
                            };
                            let wave = value.value.clone();
                            let value: T = Replay::replay(&wave_value::<T>(&value.value));
                            assert!(writer.write(value).await.is_ok(), "the guest dropped {label:?} before it took {wave}");
                        });
                        reader
                    }
                    // Reads the guest's value, and checks it, unless the trace drops the future.
                    fn consume(self) {
                        let label = label_of(&self);
                        spawn(async move {
                            match next_line(&label).await {
                                Some(_) => drop(self),
                                None => {
                                    let value = self.await;
                                    let wave = wasm_wave::to_string(&value.to_value()).unwrap();
                                    let line = guest_line(Event::Value(FutureValue { future: label, value: wave }));
                                    proxy::recorder::replay::check(&line);
                                    host_dropping(move || drop(value));
                                }
                            }
                        });
                    }
                }
            };
            items.extend(ast.items);
        }
        items
    }
}
