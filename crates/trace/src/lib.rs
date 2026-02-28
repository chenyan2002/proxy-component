use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

pub fn load_trace(reader: impl std::io::BufRead) -> Vec<FuncCall> {
    let mut res = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        match serde_json::from_str::<FuncCall>(&line) {
            Ok(item) => res.push(item),
            // Ignore non-JSON lines.
            Err(_) => continue,
        }
    }
    res
}

pub fn record_args(method: Option<String>, args: Vec<String>, is_export: bool) -> FuncCall {
    let call = if is_export {
        FuncCall::ExportArgs {
            method: method.unwrap(),
            args,
        }
    } else {
        FuncCall::ImportArgs { method, args }
    };
    call
}
pub fn record_ret(method: Option<String>, ret: Option<String>, is_export: bool) -> FuncCall {
    let call = if is_export {
        FuncCall::ExportRet { method, ret }
    } else {
        FuncCall::ImportRet { method, ret }
    };
    call
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
