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
