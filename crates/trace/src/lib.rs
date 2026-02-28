use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FuncCall {
    ExportArgs {
        method: String,
        args: Vec<String>,
    },
    ExportRet {
        method: Option<String>,
        ret: Option<String>,
    },
    ImportArgs {
        method: Option<String>,
        args: Vec<String>,
    },
    ImportRet {
        method: Option<String>,
        ret: Option<String>,
    },
}

pub struct Logger(pub VecDeque<FuncCall>);

impl Logger {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn load_trace(&mut self, text: &str) {
        self.0.clear();
        for line in text.lines() {
            match serde_json::from_str::<FuncCall>(line) {
                Ok(item) => self.0.push_back(item),
                // Ignore non-JSON lines.
                Err(_) => continue,
            }
        }
    }
    pub fn dump_trace(&self) -> String {
        self.0
            .iter()
            .map(|call| serde_json::to_string(call).unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    }
    pub fn record_args(
        &mut self,
        method: Option<String>,
        args: Vec<String>,
        is_export: bool,
    ) -> FuncCall {
        let call = if is_export {
            FuncCall::ExportArgs {
                method: method.unwrap(),
                args,
            }
        } else {
            FuncCall::ImportArgs { method, args }
        };
        self.0.push_back(call.clone());
        call
    }
    pub fn record_ret(
        &mut self,
        method: Option<String>,
        ret: Option<String>,
        is_export: bool,
    ) -> FuncCall {
        let call = if is_export {
            FuncCall::ExportRet { method, ret }
        } else {
            FuncCall::ImportRet { method, ret }
        };
        self.0.push_back(call.clone());
        call
    }
    pub fn replay_export(&mut self) -> Option<(String, Vec<String>)> {
        let call = self.0.pop_front()?;
        println!("export call: {}", call.to_string());
        let FuncCall::ExportArgs { method, args } = call else {
            panic!()
        };
        Some((method, args))
    }
    pub fn assert_export_ret(&mut self, assert_method: Option<String>, assert_ret: Option<String>) {
        if let Some(FuncCall::ExportRet { .. }) = self.0.front() {
            let call = self.0.pop_front().unwrap();
            println!("export ret: {}", call.to_string());
            let FuncCall::ExportRet { method, ret } = call else {
                panic!()
            };
            if let (Some(method), Some(assert_method)) = (method, assert_method) {
                assert_eq!(method, assert_method);
            }
            assert_eq!(ret, assert_ret);
        }
    }
    pub fn replay_import(
        &mut self,
        assert_method: Option<String>,
        assert_args: Option<Vec<String>>,
        from_guest: bool,
    ) -> (bool, Option<String>) {
        let mut exit_called = false;
        let mut call = self.0.pop_front().unwrap();
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
                if from_guest {
                    let code = if assert_args
                        .is_some_and(|args| args.get(0).is_some_and(|arg| arg.starts_with("err")))
                    {
                        1
                    } else {
                        0
                    };
                    std::process::exit(code);
                } else {
                    exit_called = true;
                    return (exit_called, Some("Something that can crash".to_string()));
                }
            }
            call = self.0.pop_front().unwrap();
        }
        println!("import ret: {}", call.to_string());
        let FuncCall::ImportRet { ret, .. } = call else {
            panic!()
        };
        (exit_called, ret)
    }
}

impl FuncCall {
    pub fn to_string(&self) -> String {
        match self {
            FuncCall::ExportArgs { method, args } => format!("{method}({})", args.join(", ")),
            FuncCall::ImportArgs { method, args } => format!(
                "{}({})",
                method.as_deref().unwrap_or("<unknown>"),
                args.join(", ")
            ),
            FuncCall::ExportRet { ret, .. } | FuncCall::ImportRet { ret, .. } => {
                ret.as_deref().unwrap_or("()").to_owned()
            }
        }
    }
}
