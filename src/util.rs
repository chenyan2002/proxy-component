use std::borrow::Cow;

pub fn ident(name: &str) -> Cow<'_, str> {
    if is_keyword(name) {
        Cow::Owned(format!("%{name}"))
    } else {
        Cow::Borrowed(name)
    }
}
// from https://docs.rs/wit-component/latest/src/wit_component/printing.rs.html#155-192
pub fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "use"
            | "type"
            | "func"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "s8"
            | "s16"
            | "s32"
            | "s64"
            | "f32"
            | "f64"
            | "float32"
            | "float64"
            | "char"
            | "resource"
            | "record"
            | "flags"
            | "variant"
            | "enum"
            | "bool"
            | "string"
            | "option"
            | "result"
            | "future"
            | "stream"
            | "list"
            | "own"
            | "borrow"
            | "_"
            | "as"
            | "from"
            | "static"
            | "interface"
            | "tuple"
            | "world"
            | "import"
            | "export"
            | "package"
            | "with"
            | "include"
            | "constructor"
            | "error-context"
            | "async"
    )
}
