use indexmap::{IndexMap, IndexSet};

use std::{borrow::Cow, collections::HashMap};

use crate::TypeScriptType;

#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
    // use IndexMap to preserve the insertion order
    types: IndexMap<&'static str, (TypeScriptType, Cow<'static, str>)>,
    tags: HashMap<&'static str, Vec<&'static str>>,
    external: IndexSet<Cow<'static, str>>,
}

pub struct DisplayRegistry<'a> {
    registry: &'a TypeRegistry,
}

impl core::fmt::Display for DisplayRegistry<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.registry.fmt(f)
    }
}

impl TypeRegistry {
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &TypeScriptType)> {
        self.types.iter().map(|(&name, (ty, _))| (name, ty))
    }

    pub fn display(&self) -> DisplayRegistry {
        DisplayRegistry { registry: self }
    }

    pub fn insert(&mut self, name: &'static str, mut ty: TypeScriptType, comment: impl Into<Cow<'static, str>>) {
        ty.unify();

        self.types.insert(name, (ty, comment.into()));
    }

    /// Adds an arbitrary string tag to a named type
    pub fn tag(&mut self, name: &'static str, tag: &'static str) {
        self.tags.entry(tag).or_default().push(name);
    }

    /// Returns all tags for the given type
    pub fn get_tags(&self, tag: &'static str) -> Option<&[&'static str]> {
        self.tags.get(tag).map(|v| &v[..])
    }

    /// Returns true if the type has the given tag
    pub fn has_tag(&self, name: &'static str, tag: &'static str) -> bool {
        self.tags.get(tag).map_or(false, |v| v.contains(&name))
    }

    /// Returns all types with the given tag
    pub fn tagged_types(&self, tag: &'static str) -> impl Iterator<Item = (&'static str, &TypeScriptType)> {
        self.tags.get(tag).into_iter().flat_map(move |v| v.iter().map(move |&name| (name, &self.types[name].0)))
    }

    pub fn type_tags(&self, ty: &'static str) -> impl Iterator<Item = &'static str> + '_ {
        self.tags.iter().filter_map(move |(tag, names)| if names.contains(&ty) { Some(*tag) } else { None })
    }

    pub fn get(&self, name: &'static str) -> Option<&TypeScriptType> {
        self.types.get(name).map(|(ty, _)| ty)
    }

    pub fn contains(&self, name: &'static str) -> bool {
        self.types.contains_key(name)
    }

    pub fn add_external(&mut self, name: impl Into<Cow<'static, str>>) {
        self.external.insert(name.into());
    }

    pub fn external(&self) -> &IndexSet<Cow<'static, str>> {
        &self.external
    }
}

use core::fmt::{Display, Error as FmtError, Write};

impl TypeScriptType {
    fn is_extendible(&self, registry: &TypeRegistry) -> bool {
        match self {
            TypeScriptType::Interface { .. } => true,
            TypeScriptType::Named(name) => match registry.get(name) {
                Some(ty) => ty.is_extendible(registry),
                None => false,
            },
            _ => false,
        }
    }
}

impl TypeRegistry {
    pub fn fmt_to_string(&self) -> Result<String, FmtError> {
        let mut out = String::new();
        self.fmt(&mut out)?;
        Ok(out)
    }

    pub fn fmt<W: Write>(&self, mut out: W) -> core::fmt::Result {
        let mut first = true;

        for (name, (ty, item_comment)) in &self.types {
            if !first {
                out.write_str("\n\n")?;
            }

            first = false;

            fmt_comment(item_comment, &mut out, "")?;

            match ty {
                // values are just exported as constants
                TypeScriptType::Boolean(Some(_)) | TypeScriptType::Number(Some(_)) | TypeScriptType::String(Some(_)) => {
                    writeln!(out, "export const {name} = {ty};")?;
                }

                TypeScriptType::EnumValue(e, v) => {
                    writeln!(out, "export const {name} = {e}.{v};")?;
                }

                // null, undefined, and basic types are just exported as types
                TypeScriptType::Boolean(None)
                | TypeScriptType::Number(None)
                | TypeScriptType::String(None)
                | TypeScriptType::Null
                | TypeScriptType::Undefined
                | TypeScriptType::Tuple(_)
                | TypeScriptType::Array(_, _)
                | TypeScriptType::Partial(_)
                | TypeScriptType::ReadOnly(_)
                | TypeScriptType::ArrayLiteral(_)
                | TypeScriptType::Named(_) => {
                    if ty.is_literal() {
                        write!(out, "export const {name} = {ty};")?;
                    } else {
                        write!(out, "export type {name} = {ty};")?;
                    }
                }

                // make sure these are unwrapped for top-level types
                TypeScriptType::Intersection(_) | TypeScriptType::Union(_) => {
                    write!(out, "export type {name} = ")?;
                    ty.fmt_depth(0, &mut out)?;
                    out.write_str(";")?;
                }

                TypeScriptType::Map(key, value) => {
                    if !key.is_key_type() {
                        eprintln!("Warning: key type for map {name} is not a key type");
                    }

                    write!(out, "export type {name} = {{ [key: {key}]: {value} }};")?;
                }

                TypeScriptType::Enum(vec) | TypeScriptType::ConstEnum(vec) => {
                    let specifier = match ty {
                        TypeScriptType::ConstEnum(_) => "const enum",
                        TypeScriptType::Enum(_) => "enum",
                        _ => unreachable!(),
                    };

                    writeln!(out, "export {specifier} {name} {{")?;
                    for (name, value, comment) in vec {
                        fmt_comment(comment, &mut out, "    ")?;

                        match value {
                            Some(value) => writeln!(out, "    {name} = {value},")?,
                            None => writeln!(out, "    {name},")?,
                        }
                    }
                    out.write_str("}")?;
                }

                TypeScriptType::Interface { members, extends } => {
                    // no members, just extends, so take an intersection of the extends
                    if members.is_empty() && !extends.is_empty() {
                        write!(out, "export type {name} = ")?;
                        for (i, extend) in extends.iter().enumerate() {
                            if i != 0 {
                                out.write_str(" & ")?;
                            }
                            write!(out, "{extend}")?;
                        }
                        out.write_str(";")?;

                        continue;
                    }

                    // only use interface extend if all extends are interfaces
                    let do_extend = extends.iter().all(|extend| match extend {
                        TypeScriptType::Named(name) => matches!(self.get(name), Some(ty) if ty.is_extendible(self)),
                        _ => false,
                    });

                    if do_extend {
                        // all extends are interfaces, so we can just extend them
                        write!(out, "export interface {name}")?;

                        if !extends.is_empty() {
                            out.write_str(" extends ")?;
                            for (i, extend) in extends.iter().enumerate() {
                                if i != 0 {
                                    out.write_str(", ")?;
                                }
                                write!(out, "{extend}")?;
                            }
                        }
                    } else {
                        // take the intersection of the interface and extends
                        write!(out, "export type {name} = ")?;

                        for extend in extends {
                            write!(out, "{extend} &")?;
                        }
                    }

                    out.write_str(" {\n")?;
                    for (name, ty, member_comment) in members {
                        let ty = ty.take_optional();

                        let (opt, ty) = match ty {
                            Ok(ref ty) => ("?", ty),
                            Err(ty) => ("", ty),
                        };

                        fmt_comment(member_comment, &mut out, "    ")?;

                        write!(out, "    {name}{opt}: ")?;
                        ty.fmt_depth(0, &mut out)?;
                        out.write_str(",\n")?;
                    }
                    out.write_str("}")?;

                    if !do_extend {
                        out.write_str(";")?;
                    }
                }
                TypeScriptType::ApiDecl {
                    command_flags,
                    name,
                    method,
                    form_type,
                    return_type,
                    body_type,
                    path,
                } => {
                    let body_type = match body_type {
                        Some(ty) => ty,
                        None => &TypeScriptType::Null,
                    };

                    writeln!(
                        out,
                        "export const {name} = /*#__PURE__*/command.{}<{form_type}, {return_type}, {body_type}>({{",
                        method.to_lowercase()
                    )?;

                    if path.contains("${") {
                        writeln!(out, "    path() {{ return `{path}`; }},")?;
                    } else {
                        writeln!(out, "    path: \"{path}\",")?;
                    }

                    if !command_flags.is_empty() {
                        write!(out, "    flags: ")?;
                        for (idx, flag) in command_flags.iter().enumerate() {
                            if idx != 0 {
                                write!(out, " | ")?;
                            }
                            write!(out, "{flag}")?;
                        }

                        writeln!(out, ",")?;
                    }

                    out.write_str("});")?;
                }
            }
        }

        Ok(())
    }
}

impl TypeScriptType {
    fn fmt_depth<W: Write>(&self, depth: usize, f: &mut W) -> std::fmt::Result {
        match self {
            TypeScriptType::Named(name) => f.write_str(name),
            TypeScriptType::ApiDecl { name, .. } => f.write_str(name),

            TypeScriptType::Null => f.write_str("null"),
            TypeScriptType::Undefined => f.write_str("undefined"),

            TypeScriptType::EnumValue(e, v) => write!(f, "{e}.{v}"),

            TypeScriptType::Array(inner, _) => write!(f, "Array<{inner}>"),
            TypeScriptType::Partial(inner) => write!(f, "Partial<{inner}>"),
            TypeScriptType::ReadOnly(inner) => {
                if inner.is_literal() {
                    write!(f, "{inner} as const")
                } else {
                    write!(f, "Readonly<{inner}>")
                }
            }

            TypeScriptType::ArrayLiteral(vec) => {
                f.write_str("[")?;

                if vec.len() > 5 {
                    f.write_str("\n    ")?;
                }

                for (i, ty) in vec.iter().enumerate() {
                    if i != 0 {
                        if i % 5 == 0 {
                            f.write_str(",\n    ")?;
                        } else {
                            f.write_str(", ")?;
                        }
                    }
                    ty.fmt_depth(depth + 1, f)?;
                }

                if vec.len() > 5 {
                    f.write_str("\n")?;
                }

                f.write_str("]")
            }

            TypeScriptType::Boolean(value) => match value {
                Some(value) => write!(f, "{value}"),
                None => f.write_str("boolean"),
            },
            TypeScriptType::Number(value) => match value {
                Some(value) => write!(f, "{value}"),
                None => f.write_str("number"),
            },
            TypeScriptType::String(value) => match value {
                Some(value) => write!(f, "\"{value}\""),
                None => f.write_str("string"),
            },

            TypeScriptType::Map(key, value) => write!(f, "{{ [key: {key}]: {value} }}"),

            TypeScriptType::Union(vec) | TypeScriptType::Intersection(vec) => {
                let big_decl = depth == 0 && vec.len() > 2;

                let joiner = match (self, big_decl) {
                    (TypeScriptType::Union(_), false) => " | ",
                    (TypeScriptType::Intersection(_), false) => " & ",
                    (TypeScriptType::Union(_), true) => "\n    | ",
                    (TypeScriptType::Intersection(_), true) => "\n    & ",
                    _ => unreachable!(),
                };

                if big_decl {
                    f.write_str(joiner)?;
                }

                if depth > 1 && vec.len() > 1 {
                    f.write_str("(")?;
                }

                for (i, ty) in vec.iter().enumerate() {
                    if i != 0 {
                        f.write_str(joiner)?;
                    }
                    ty.fmt_depth(depth + 1, f)?;
                }

                if depth > 1 && vec.len() > 1 {
                    f.write_str(")")?;
                }

                Ok(())
            }

            TypeScriptType::Tuple(vec) => {
                f.write_str("[")?;
                for (i, (ty, element_comment)) in vec.iter().enumerate() {
                    if i != 0 {
                        f.write_str(", ")?;
                    }

                    fmt_comment(element_comment, f, "")?;

                    ty.fmt_depth(depth + 1, f)?;
                }
                f.write_str("]")
            }

            TypeScriptType::ConstEnum(_) | TypeScriptType::Enum(_) => {
                panic!("Enums should be handled by TypeRegistry");
            }

            TypeScriptType::Interface { members, extends } => {
                f.write_str("{ ")?;
                for (name, ty, member_comment) in members.iter() {
                    let ty = ty.take_optional();

                    let (opt, ty) = match ty {
                        Ok(ref ty) => ("?", ty),
                        Err(ty) => ("", ty),
                    };

                    fmt_comment(member_comment, f, "")?;

                    write!(f, "{name}{opt}: ")?;
                    ty.fmt_depth(0, f)?;
                    f.write_str(", ")?;
                }

                f.write_str("}")?;

                for extend in extends {
                    write!(f, "& {extend}")?;
                }

                Ok(())
            }
        }
    }
}

impl Display for TypeScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_depth(1, f)
    }
}

fn fmt_comment<W: Write>(comment: &str, out: &mut W, p: &str) -> std::fmt::Result {
    if comment.is_empty() {
        return Ok(());
    }

    if !comment.contains('\n') {
        return writeln!(out, "{p}/** {} */", comment.trim());
    }

    writeln!(out, "{p}/**")?;

    for line in comment.lines() {
        writeln!(out, "{p} * {}", line.trim())?;
    }

    writeln!(out, "{p} */")
}
