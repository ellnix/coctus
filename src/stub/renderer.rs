use tera::{Tera, Context};
use tera_text_filters::{snake_case, camel_case, mixed_case};

use crate::programming_language::{ProgrammingLanguage, VariableNameFormat};
use super::parser::{Cmd, Stub, VariableCommand};

mod types;
use types::ReadData;

pub fn render_stub(lang: ProgrammingLanguage, stub: Stub) -> String {
    let rend = Renderer::new(lang, stub);
    rend.render()
}

struct Renderer {
    tera: Tera,
    lang: ProgrammingLanguage,
    stub: Stub,
}

impl Renderer {
    fn new(lang: ProgrammingLanguage, stub: Stub) -> Self {
        let mut tera = Tera::new(&lang.template_glob())
            .expect("There are no templates for this language");
        let case_fn = match lang.variable_format {
            // camel_case types don't match tera-text-filters conventions. To fix.
            VariableNameFormat::SnakeCase => snake_case,
            VariableNameFormat::CamelCase => mixed_case,
            VariableNameFormat::PascalCase => camel_case,
        };
        tera.register_filter("case", case_fn);
        Self { lang, tera, stub }
    }

    fn render(&self) -> String {
        let mut context = Context::new();

        // Transform self.stub.commands into successive strings
        let commands: Vec<String> = self.stub.commands.iter().map(|cmd| {
            let cmd_str = self.render_command(cmd);
            // TODO: Make this less stupid
            format!("{}\n", cmd_str.as_str()).replace("\n\n", "\n").replace("\n\n", "\n")
        }).collect();

        context.insert("commands", &commands);

        self.tera.render(&format!("main.{}.jinja", self.lang.source_file_ext), &context)
            .expect("Failed to render template for stub")
    }

    fn render_command(&self, cmd: &Cmd) -> String {
        match cmd {
            Cmd::Read(vars) => self.render_read(vars),
            Cmd::Write(message) => self.render_write(message),
            Cmd::Loop { count, command } => self.render_loop(count, command),
            Cmd::LoopLine { object, variables } => self.render_loopline(object, variables),
        }
    }

    fn render_write(&self, message: &String) -> String {
        let mut context = Context::new();
        context.insert("messages", &message.lines().collect::<Vec<&str>>());
        self.tera.render(&self.template_path("write"), &context)
            .expect("Could not find write template")
    }

    fn render_read(&self, vars: &Vec<VariableCommand>) -> String {
        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();
        let mut context = Context::new();
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        self.tera.render(&self.template_path("read"), &context)
            .expect("Could not find read template").trim_end().to_owned()
    }

    fn render_loop(&self, count: &String, cmd: &Box<Cmd>) -> String {
        let mut context = Context::new();
        let rendered_cmd: Vec<String> = self.render_command(&cmd).lines().map(|s|s.to_owned()).collect();
        context.insert("count", &count);
        context.insert("inner", &rendered_cmd);
        self.tera.render(&self.template_path("loop"), &context)
            .expect("Could not find loop template")
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{template_name}.{}.jinja", self.lang.source_file_ext)
    }

    fn render_loopline(&self, object: &str, vars: &Vec<VariableCommand>) -> String {
        let read_data: Vec<ReadData> = vars.into_iter().map(|var_cmd| ReadData::from(var_cmd)).collect();
        let mut context = Context::new();
        context.insert("object", &object);
        context.insert("vars", &read_data);
        context.insert("type_tokens", &self.lang.type_tokens);
        self.tera.render(&self.template_path("loopline"), &context)
            .expect("Could not find read template")
    }
}



