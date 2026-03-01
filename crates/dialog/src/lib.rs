use console::{Style, StyledObject, style};
use dialoguer::{Input, Select, theme::Theme};
use std::fmt;
use wasm_wave::value::convert::ToValue;

pub fn print(message: &str) {
    let theme = IndentTheme::new(0);
    theme.println(message);
}

pub fn read_string(dep: u32) -> String {
    let theme = IndentTheme::new(dep as usize);
    let text = Input::<String>::with_theme(&theme)
        .allow_empty(true)
        .with_prompt("Enter a string")
        .interact()
        .unwrap();
    wasm_wave::to_string(&text.to_value()).unwrap()
}
pub fn read_bool(dep: u32) -> String {
    let theme = IndentTheme::new(dep as usize);
    let selection = Select::with_theme(&theme)
        .with_prompt("Select a bool")
        .items(&["true", "false"])
        .interact()
        .unwrap();
    let value = if selection == 0 { true } else { false };
    wasm_wave::to_string(&value.to_value()).unwrap()
}
macro_rules! read_primitive {
    ($fn_name:ident, $ty:ty, $prompt:expr) => {
        pub fn $fn_name(dep: u32) -> String {
            let theme = IndentTheme::new(dep as usize);
            let num = Input::<$ty>::with_theme(&theme)
                .with_prompt($prompt)
                .interact_text()
                .unwrap();
            wasm_wave::to_string(&num.to_value()).unwrap()
        }
    };
}

read_primitive!(read_u8, u8, "Enter a u8");
read_primitive!(read_u16, u16, "Enter a u16");
read_primitive!(read_u32, u32, "Enter a u32");
read_primitive!(read_u64, u64, "Enter a u64");
read_primitive!(read_s8, i8, "Enter a s8");
read_primitive!(read_s16, i16, "Enter a s16");
read_primitive!(read_s32, i32, "Enter a s32");
read_primitive!(read_s64, i64, "Enter a s64");
read_primitive!(read_f32, f32, "Enter a f32");
read_primitive!(read_f64, f64, "Enter a f64");
read_primitive!(read_char, char, "Enter a char");

pub struct IndentTheme {
    indent: usize,
    defaults_style: Style,
    prompt_style: Style,
    prompt_prefix: StyledObject<String>,
    prompt_suffix: StyledObject<String>,
    success_prefix: StyledObject<String>,
    success_suffix: StyledObject<String>,
    error_prefix: StyledObject<String>,
    error_style: Style,
    hint_style: Style,
    values_style: Style,
    active_item_style: Style,
    inactive_item_style: Style,
    active_item_prefix: StyledObject<String>,
    inactive_item_prefix: StyledObject<String>,
}
impl IndentTheme {
    pub fn new(indent: usize) -> Self {
        Self {
            indent,
            defaults_style: Style::new().for_stderr().cyan(),
            prompt_style: Style::new().for_stderr().bold(),
            prompt_prefix: style("?".to_string()).for_stderr().yellow(),
            prompt_suffix: style("›".to_string()).for_stderr().black().bright(),
            success_prefix: style("✔".to_string()).for_stderr().green(),
            success_suffix: style("·".to_string()).for_stderr().black().bright(),
            error_prefix: style("✘".to_string()).for_stderr().red(),
            error_style: Style::new().for_stderr().red(),
            hint_style: Style::new().for_stderr().black().bright(),
            values_style: Style::new().for_stderr().green(),
            active_item_style: Style::new().for_stderr().cyan(),
            inactive_item_style: Style::new().for_stderr(),
            active_item_prefix: style("❯".to_string()).for_stderr().green(),
            inactive_item_prefix: style(" ".to_string()).for_stderr(),
        }
    }
    pub fn indent(&self, f: &mut dyn fmt::Write) -> fmt::Result {
        let spaces = " ".repeat(self.indent * 2);
        write!(f, "{spaces}")
    }
    pub fn println(&self, prompt: &str) {
        let spaces = " ".repeat(self.indent * 2);
        println!("{spaces}{prompt}");
    }
    pub fn hint(&self, prompt: &str) {
        let spaces = " ".repeat(self.indent * 2);
        println!("{spaces}{}", self.hint_style.apply_to(prompt));
    }
}
impl Theme for IndentTheme {
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        self.indent(f)?;
        write!(
            f,
            "{} {} ",
            &self.prompt_prefix,
            self.prompt_style.apply_to(prompt)
        )?;
        write!(f, "{}", &self.prompt_suffix)
    }
    fn format_error(&self, f: &mut dyn fmt::Write, err: &str) -> fmt::Result {
        self.indent(f)?;
        write!(
            f,
            "{} {}",
            &self.error_prefix,
            self.error_style.apply_to(err)
        )
    }
    fn format_input_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<&str>,
    ) -> fmt::Result {
        self.indent(f)?;
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        match default {
            Some(default) => write!(
                f,
                "{} {} ",
                self.hint_style.apply_to(&format!("({})", default)),
                &self.prompt_suffix
            ),
            None => write!(f, "{} ", &self.prompt_suffix),
        }
    }
    fn format_confirm_prompt(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        default: Option<bool>,
    ) -> fmt::Result {
        self.indent(f)?;
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        match default {
            None => write!(
                f,
                "{} {}",
                self.hint_style.apply_to("(y/n)"),
                &self.prompt_suffix
            ),
            Some(true) => write!(
                f,
                "{} {} {}",
                self.hint_style.apply_to("(y/n)"),
                &self.prompt_suffix,
                self.defaults_style.apply_to("yes")
            ),
            Some(false) => write!(
                f,
                "{} {} {}",
                self.hint_style.apply_to("(y/n)"),
                &self.prompt_suffix,
                self.defaults_style.apply_to("no")
            ),
        }
    }
    fn format_confirm_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        selection: Option<bool>,
    ) -> fmt::Result {
        self.indent(f)?;
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }
        let selection = selection.map(|b| if b { "yes" } else { "no" });

        match selection {
            Some(selection) => {
                write!(
                    f,
                    "{} {}",
                    &self.success_suffix,
                    self.values_style.apply_to(selection)
                )
            }
            None => {
                write!(f, "{}", &self.success_suffix)
            }
        }
    }
    fn format_input_prompt_selection(
        &self,
        f: &mut dyn fmt::Write,
        prompt: &str,
        sel: &str,
    ) -> fmt::Result {
        self.indent(f)?;
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.success_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        write!(
            f,
            "{} {}",
            &self.success_suffix,
            self.values_style.apply_to(sel)
        )
    }
    fn format_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        active: bool,
    ) -> fmt::Result {
        self.indent(f)?;
        let details = if active {
            (
                &self.active_item_prefix,
                self.active_item_style.apply_to(text),
            )
        } else {
            (
                &self.inactive_item_prefix,
                self.inactive_item_style.apply_to(text),
            )
        };

        write!(f, "{} {}", details.0, details.1)
    }
}
