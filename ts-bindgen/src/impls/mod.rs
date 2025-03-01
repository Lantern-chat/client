use super::{TypeRegistry, TypeScriptDef, TypeScriptType};

mod alloc_impl;
mod core_impl;
mod std_impl;

impl TypeScriptDef for snowflake::Snowflake {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        // defined externally
        registry.add_external("Snowflake");
        TypeScriptType::Named("Snowflake")
    }
}

impl TypeScriptDef for timestamp::Timestamp {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        // defined externally
        registry.add_external("Timestamp");
        TypeScriptType::Named("Timestamp")
    }
}

impl TypeScriptDef for smol_str::SmolStr {
    fn register(_registry: &mut TypeRegistry) -> TypeScriptType {
        TypeScriptType::string()
    }
}

impl<T: TypeScriptDef> TypeScriptDef for thin_vec::ThinVec<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry).into_array()
    }
}

impl<T: TypeScriptDef> TypeScriptDef for triomphe::Arc<T> {
    fn register(registry: &mut TypeRegistry) -> TypeScriptType {
        T::register(registry)
    }
}
