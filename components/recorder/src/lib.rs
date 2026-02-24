mod bindings {
    wit_bindgen::generate!({
        path: "../../assets/recorder.wit",
        world: "guest",
    });
}
mod trace;

use trace::FuncCall;
struct Component;
impl bindings::exports::proxy::recorder::record::Guest for Component {
    fn record_args(method: Option<String>, args: Vec<String>, is_export: bool) {
        let call = if is_export {
            FuncCall::ExportArgs {
                method: method.unwrap(),
                args,
            }
        } else {
            FuncCall::ImportArgs { method, args }
        };
        let json = serde_json::to_string(&call).unwrap();
        println!("{json}");
    }
    fn record_ret(method: Option<String>, ret: Option<String>, is_export: bool) {
        let call = if is_export {
            FuncCall::ExportRet { method, ret }
        } else {
            FuncCall::ImportRet { method, ret }
        };
        let json = serde_json::to_string(&call).unwrap();
        println!("{json}");
    }
}

use std::cell::RefCell;
use std::collections::VecDeque;
thread_local! {
    static TRACE: RefCell<Option<VecDeque<FuncCall>>> = RefCell::new(None);
}

fn load_trace() {
    let load = TRACE.with_borrow(|v| v.is_none());
    if load {
        TRACE.with_borrow_mut(|v| {
            use std::io::BufRead;
            let mut res = VecDeque::new();
            let f = std::io::stdin();
            let reader = std::io::BufReader::new(f);
            for line in reader.lines() {
                let line = line.unwrap();
                match serde_json::from_str::<FuncCall>(&line) {
                    Ok(item) => res.push_back(item),
                    Err(_) => continue,
                }
            }
            *v = Some(res);
        });
    }
}

impl bindings::exports::proxy::recorder::replay::Guest for Component {
    fn replay_export() -> Option<(String, Vec<String>)> {
        load_trace();
        TRACE.with_borrow_mut(|v| {
            let call = v.as_mut().unwrap().pop_front()?;
            println!("export call: {}", call.to_string());
            let FuncCall::ExportArgs { method, args } = call else {
                panic!()
            };
            Some((method, args))
        })
    }
    fn assert_export_ret(assert_method: Option<String>, assert_ret: Option<String>) {
        TRACE.with_borrow_mut(|v| {
            if let Some(FuncCall::ExportRet { .. }) = v.as_mut().unwrap().front() {
                let call = v.as_mut().unwrap().pop_front().unwrap();
                println!("export ret: {}", call.to_string());
                let FuncCall::ExportRet { method, ret } = call else {
                    panic!()
                };
                if let (Some(method), Some(assert_method)) = (method, assert_method) {
                    assert_eq!(method, assert_method);
                }
                assert_eq!(ret, assert_ret);
            }
        });
    }
    fn replay_import(
        assert_method: Option<String>,
        assert_args: Option<Vec<String>>,
    ) -> Option<String> {
        TRACE.with_borrow_mut(|v| {
            let mut call = v.as_mut().unwrap().pop_front().unwrap();
            if let FuncCall::ImportArgs { method, args } = &call {
                if let (Some(method), Some(assert_method)) = (method, assert_method) {
                    assert_eq!(method, &assert_method);
                }
                if let Some(assert_args) = &assert_args {
                    assert_eq!(args, assert_args);
                }
                println!("import call: {}", call.to_string());
                if method
                    .as_ref()
                    .is_some_and(|m| m.starts_with("wasi:cli/exit"))
                {
                    let code = if assert_args
                        .is_some_and(|args| args.get(0).is_some_and(|arg| arg.starts_with("err")))
                    {
                        1
                    } else {
                        0
                    };
                    std::process::exit(code);
                }
                call = v.as_mut().unwrap().pop_front().unwrap();
            }
            println!("import ret: {}", call.to_string());
            let FuncCall::ImportRet { ret, .. } = call else {
                panic!()
            };
            ret
        })
    }
}
bindings::export!(Component with_types_in bindings);
