// RGB standard library for working with smart contracts on Bitcoin & Lightning
//
// SPDX-License-Identifier: Apache-2.0
//
// Written in 2019-2024 by
//     Dr Maxim Orlovsky <orlovsky@lnp-bp.org>
//
// Copyright (C) 2019-2024 LNP/BP Standards Association. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

use amplify::confinement::{TinyOrdMap, TinyOrdSet, TinyString};
use rgb::Occurrences;
use strict_encoding::{FieldName, TypeName, Variant};
use strict_types::{SemId, SymbolicSys};

use super::{
    ArgMap, ExtensionIface, GenesisIface, Iface, IfaceId, Modifier, OwnedIface, TransitionIface,
};

struct ArgMapDisplay<'a>(&'a ArgMap);

impl<'a> Display for ArgMapDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (i, (name, occ)) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?
            }
            write!(f, "{name}")?;
            match occ {
                Occurrences::Once => Ok(()),
                Occurrences::NoneOrOnce => write!(f, "(?)"),
                Occurrences::NoneOrMore => write!(f, "(*)"),
                Occurrences::OnceOrMore => write!(f, "(+)"),
                Occurrences::NoneOrUpTo(to) => write!(f, "(..{to})"),
                Occurrences::OnceOrUpTo(to) => write!(f, "(1..{to})"),
                Occurrences::Exactly(v) => write!(f, "({v})"),
                Occurrences::Range(r) => write!(f, "({}..{})", r.start(), r.end()),
            }?;
        }
        Ok(())
    }
}

struct OpIfaceDisplay<'a> {
    metadata: Option<SemId>,
    globals: &'a ArgMap,
    assignments: &'a ArgMap,
    valencies: &'a TinyOrdSet<FieldName>,
    errors: &'a TinyOrdSet<u8>,
    types: &'a SymbolicSys,
    iface_errors: &'a TinyOrdMap<Variant, TinyString>,
}

impl<'a> OpIfaceDisplay<'a> {
    fn genesis(op: &'a GenesisIface, iface: &'a IfaceDisplay) -> Self {
        Self {
            metadata: op.metadata,
            globals: &op.globals,
            assignments: &op.assignments,
            valencies: &op.valencies,
            errors: &op.errors,
            types: iface.types,
            iface_errors: &iface.iface.errors,
        }
    }

    fn transition(op: &'a TransitionIface, iface: &'a IfaceDisplay) -> Self {
        Self {
            metadata: op.metadata,
            globals: &op.globals,
            assignments: &op.assignments,
            valencies: &op.valencies,
            errors: &op.errors,
            types: iface.types,
            iface_errors: &iface.iface.errors,
        }
    }

    fn extension(op: &'a ExtensionIface, iface: &'a IfaceDisplay) -> Self {
        Self {
            metadata: op.metadata,
            globals: &op.globals,
            assignments: &op.assignments,
            valencies: &op.valencies,
            errors: &op.errors,
            types: iface.types,
            iface_errors: &iface.iface.errors,
        }
    }
}

impl<'a> Display for OpIfaceDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !self.iface_errors.is_empty() {
            write!(f, "\t\terrors: ")?;
            for (i, tag) in self.errors.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                match self.iface_errors.keys().find(|v| v.tag == *tag) {
                    Some(var) => write!(f, "{var}")?,
                    None => write!(f, "{tag}")?,
                }
            }
            writeln!(f)?;
        }

        if let Some(meta_id) = self.metadata {
            write!(f, "\t\tmeta: ")?;
            match self.types.lookup(meta_id) {
                Some(fqn) => writeln!(f, "{fqn}"),
                None => writeln!(f, "{meta_id} -- type name is unknown"),
            }?;
        }
        if !self.globals.is_empty() {
            writeln!(f, "\t\tglobals: {}", ArgMapDisplay(self.globals))?;
        }
        if !self.valencies.is_empty() {
            write!(f, "\t\tvalencies: ")?;
            for (i, name) in self.valencies.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?
                }
                write!(f, "{name}")?;
            }
            writeln!(f)?;
        }
        if !self.assignments.is_empty() {
            writeln!(f, "\t\tassigns: {}", ArgMapDisplay(self.assignments))?;
        }
        Ok(())
    }
}

pub struct IfaceDisplay<'a> {
    iface: &'a Iface,
    externals: HashMap<IfaceId, &'a TypeName>,
    types: &'a SymbolicSys,
}

impl<'a> IfaceDisplay<'a> {
    pub fn new(
        iface: &'a Iface,
        externals: HashMap<IfaceId, &'a TypeName>,
        types: &'a SymbolicSys,
    ) -> Self {
        Self {
            iface,
            types,
            externals,
        }
    }
}

impl<'a> Display for IfaceDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        fn sugar(f: &mut Formatter<'_>, required: bool, multiple: bool) -> fmt::Result {
            match (required, multiple) {
                (true, true) => write!(f, "(+)"),
                (false, false) => write!(f, "(?)"),
                (false, true) => write!(f, "(*)"),
                _ => Ok(()),
            }
        }
        fn resolve(f: &mut Formatter<'_>, types: &SymbolicSys, id: SemId) -> fmt::Result {
            match types.lookup(id) {
                Some(fqn) => write!(f, "{fqn}"),
                None => write!(f, "{id:-} -- type name unknown"),
            }
        }
        fn opsugar(
            f: &mut Formatter<'_>,
            pred: &str,
            name: Option<&FieldName>,
            modifier: Modifier,
            optional: bool,
            default: bool,
        ) -> fmt::Result {
            write!(f, "\t{pred}")?;
            if let Some(name) = name {
                write!(f, " {name}")?;
            }
            let mut modifiers = vec![];
            if !optional {
                modifiers.push("required");
            }
            if default {
                modifiers.push("default");
            }
            match modifier {
                Modifier::Final => modifiers.push("final"),
                Modifier::Abstract => modifiers.push("abstract"),
                Modifier::Override => modifiers.push("override"),
            }

            if !modifiers.is_empty() {
                f.write_str(": ")?;
            }
            for (i, name) in modifiers.into_iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?;
                }
                f.write_str(name)?;
            }
            writeln!(f)
        }

        writeln!(f, "@version({})", self.iface.version)?;
        write!(f, "interface {}", self.iface.name)?;
        if !self.externals.is_empty() {
            f.write_str(": ")?;
            for (index, id) in self.iface.inherits.iter().enumerate() {
                if index > 0 {
                    f.write_str(", ")?;
                }
                match self.externals.get(id) {
                    Some(name) => write!(f, "{name}")?,
                    None => writeln!(f, "{id:-}")?,
                }
            }
        }
        writeln!(f)?;

        for (fname, g) in &self.iface.global_state {
            write!(f, "\tglobal {fname}")?;
            sugar(f, g.required, g.multiple)?;
            write!(f, ": ")?;
            match g.sem_id {
                Some(id) => resolve(f, self.types, id)?,
                None => write!(f, "Any")?,
            }
            writeln!(f)?;
        }
        writeln!(f)?;

        for (fname, a) in &self.iface.assignments {
            write!(f, "\t")?;
            match a.public {
                true => write!(f, "public ")?,
                false => write!(f, "owned ")?,
            }
            write!(f, "{fname}")?;
            sugar(f, a.required, a.multiple)?;
            f.write_str(": ")?;
            match a.owned_state {
                OwnedIface::Any => write!(f, "AnyType")?,
                OwnedIface::Amount => write!(f, "Zk64")?,
                OwnedIface::AnyData => write!(f, "Any")?,
                OwnedIface::AnyAttach => write!(f, "AnyAttachment")?,
                OwnedIface::Rights => write!(f, "Rights")?,
                OwnedIface::Data(id) => resolve(f, self.types, id)?,
            }
            writeln!(f)?;
        }
        if !self.iface.assignments.is_empty() {
            writeln!(f)?;
        }

        for (fname, v) in &self.iface.valencies {
            write!(f, "\tvalency {fname}")?;
            if !v.required {
                write!(f, "(?)")?;
            }
            writeln!(f)?;
        }
        if !self.iface.valencies.is_empty() {
            writeln!(f)?;
        }

        for (variant, descr) in &self.iface.errors {
            writeln!(f, "\terror {}: {}", variant.name, variant.tag)?;
            writeln!(f, "\t\t\"{descr}\"")?;
        }
        if !self.iface.errors.is_empty() {
            writeln!(f)?;
        }

        let op = OpIfaceDisplay::genesis(&self.iface.genesis, self);
        opsugar(f, "genesis", None, self.iface.genesis.modifier, true, false)?;
        writeln!(f, "{op}")?;

        for (name, t) in &self.iface.transitions {
            let default = self.iface.default_operation.as_ref() == Some(name);
            opsugar(f, "transition", Some(name), t.modifier, t.optional, default)?;

            let op = OpIfaceDisplay::transition(t, self);
            write!(f, "{op}")?;

            if let Some(ref d) = t.default_assignment {
                writeln!(f, "\t\tdefault: {d}")?;
            }

            writeln!(f, "\t\tinputs: {}", ArgMapDisplay(&t.inputs))?;

            writeln!(f)?;
        }

        for (name, e) in &self.iface.extensions {
            let default = self.iface.default_operation.as_ref() == Some(name);
            opsugar(f, "extension", Some(name), e.modifier, e.optional, default)?;

            let op = OpIfaceDisplay::extension(e, self);
            write!(f, "{op}")?;

            if let Some(ref d) = e.default_assignment {
                writeln!(f, "\t\tdefault: {d}")?;
            }

            write!(f, "\t\tredeems: ")?;
            for (i, name) in e.redeems.iter().enumerate() {
                if i > 0 {
                    f.write_str(", ")?
                }
                write!(f, "{name}")?;
            }
            writeln!(f)?;

            writeln!(f)?;
        }

        Ok(())
    }
}
