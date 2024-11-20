use std::io::Write as _;

use ts_bindgen::{TypeRegistry, TypeScriptDef, TypeScriptType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = TypeRegistry::default();

    client_sdk::models::gateway::message::ServerMsg::register(&mut registry);
    client_sdk::models::gateway::message::ClientMsg::register(&mut registry);

    client_sdk::api::commands::register_routes(&mut registry);

    let mut models = std::fs::File::create("autogenerated.ts")?;

    write!(models, "import type {{ ")?;

    for (idx, name) in registry.external().iter().enumerate() {
        if idx > 0 {
            write!(models, ", ")?;
        }

        write!(models, "{name}")?;
    }

    write!(
        models,
        " }} from './models';\nimport {{ command }} from './api';\n\n{}",
        registry.display()
    )?;

    let mut api = std::fs::File::create("api.ts")?;

    for group in ["decl", "values", "types"] {
        let mut first = true;
        let mut len = 0;


        if group == "types" {
            writeln!(api, "export type {{")?;
        } else {
            writeln!(api, "export {{")?;
        }

        for (name, ty) in registry.iter() {
            match group {
                "decl" if matches!(ty, TypeScriptType::ApiDecl { .. }) => {}
                "values" if ty.is_value() && !matches!(ty, TypeScriptType::ApiDecl { .. }) => {}
                "types" if !ty.is_value() => {}
                _ => continue,
            }

            if !first {
                if len % 5 == 0 {
                    write!(api, ",\n    ")?;
                } else {
                    write!(api, ", ")?;
                }
            } else {
                write!(api, "    ")?;
            }

            first = false;

            write!(api, "{}", name)?;

            len += 1;
        }

        write!(api, "\n}} from './autogenerated';\n\n")?;
    }

    Ok(())
}
