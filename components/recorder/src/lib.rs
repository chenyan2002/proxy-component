mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "guest",
    });
}

struct Component;
impl bindings::exports::proxy::recorder::record::Guest for Component {
    fn record_args(method: Option<String>, args: Vec<String>, is_export: bool) {
        if is_export {
            println!("export: {}({})", method.unwrap(), args.join(","));
        } else {
            println!(
                "import: {}({})",
                method.unwrap_or("<unknown>".to_string()),
                args.join(",")
            );
        }
    }
    fn record_ret(method: Option<String>, ret: Option<String>, is_export: bool) {
        let method = method.unwrap_or("<unknown>".to_string());
        let ret = ret.unwrap_or("()".to_string());
        if is_export {
            println!("export: {} -> {}", method, ret);
        } else {
            println!("import: {} -> {}", method, ret);
        }
    }
}
impl bindings::exports::proxy::recorder::replay::Guest for Component {
    fn replay_export() -> Option<(String, Vec<String>)> {
        None
    }
    fn assert_export_ret(_assert_method: Option<String>, _assert_ret: Option<String>) {}
    fn replay_import(
        _assert_method: Option<String>,
        _assert_args: Option<Vec<String>>,
    ) -> Option<String> {
        None
    }
}
bindings::export!(Component with_types_in bindings);
