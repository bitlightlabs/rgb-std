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

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use amplify::confinement::{TinyOrdMap, TinyOrdSet, TinyString};
use amplify::{ByteArray, Bytes32};
use baid58::{Baid58ParseError, Chunking, FromBaid58, ToBaid58, CHUNKING_32};
use commit_verify::{CommitId, CommitmentId, DigestExt, Sha256};
use rgb::{Occurrences, Types};
use strict_encoding::{
    FieldName, StrictDecode, StrictDeserialize, StrictDumb, StrictEncode, StrictSerialize,
    StrictType, TypeName, Variant,
};
use strict_types::{SemId, SymbolicSys};

use crate::interface::{IfaceDisplay, VerNo};
use crate::LIB_NAME_RGB_STD;

/// Interface identifier.
///
/// Interface identifier commits to all the interface data.
#[derive(Wrapper, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
#[wrapper(Deref, BorrowSlice, Hex, Index, RangeOps)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct IfaceId(
    #[from]
    #[from([u8; 32])]
    Bytes32,
);

impl From<Sha256> for IfaceId {
    fn from(hasher: Sha256) -> Self { hasher.finish().into() }
}

impl CommitmentId for IfaceId {
    const TAG: &'static str = "urn:lnp-bp:rgb:interface#2024-02-04";
}

impl ToBaid58<32> for IfaceId {
    const HRI: &'static str = "if";
    const CHUNKING: Option<Chunking> = CHUNKING_32;
    fn to_baid58_payload(&self) -> [u8; 32] { self.to_byte_array() }
    fn to_baid58_string(&self) -> String { self.to_string() }
}
impl FromBaid58<32> for IfaceId {}
impl Display for IfaceId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            f.write_str("urn:lnp-bp:if:")?;
        }
        if f.sign_minus() {
            write!(f, "{:.2}", self.to_baid58())
        } else {
            write!(f, "{:#.2}", self.to_baid58())
        }
    }
}
impl FromStr for IfaceId {
    type Err = Baid58ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_baid58_maybe_chunked_str(s.trim_start_matches("urn:lnp-bp:"), ':', '#')
    }
}
impl IfaceId {
    pub const fn from_array(id: [u8; 32]) -> Self { IfaceId(Bytes32::from_array(id)) }
    pub fn to_mnemonic(&self) -> String { self.to_baid58().mnemonic() }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Req {
    Optional,
    Required,
    NoneOrMore,
    OneOrMore,
}

impl Req {
    pub fn is_required(self) -> bool { self == Req::Required || self == Req::OneOrMore }
    pub fn is_multiple(self) -> bool { self == Req::NoneOrMore || self == Req::OneOrMore }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct ValencyIface {
    pub required: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct GlobalIface {
    pub sem_id: Option<SemId>,
    pub required: bool,
    pub multiple: bool,
}

impl GlobalIface {
    pub fn any(req: Req) -> Self {
        GlobalIface {
            sem_id: None,
            required: req.is_required(),
            multiple: req.is_multiple(),
        }
    }
    pub fn optional(sem_id: SemId) -> Self {
        GlobalIface {
            sem_id: Some(sem_id),
            required: false,
            multiple: false,
        }
    }
    pub fn required(sem_id: SemId) -> Self {
        GlobalIface {
            sem_id: Some(sem_id),
            required: true,
            multiple: false,
        }
    }
    pub fn none_or_many(sem_id: SemId) -> Self {
        GlobalIface {
            sem_id: Some(sem_id),
            required: false,
            multiple: true,
        }
    }
    pub fn one_or_many(sem_id: SemId) -> Self {
        GlobalIface {
            sem_id: Some(sem_id),
            required: true,
            multiple: true,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD, tags = order)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct AssignIface {
    pub owned_state: OwnedIface,
    pub public: bool,
    pub required: bool,
    pub multiple: bool,
}

impl AssignIface {
    pub fn public(owned_state: OwnedIface, req: Req) -> Self {
        AssignIface {
            owned_state,
            public: true,
            required: req.is_required(),
            multiple: req.is_multiple(),
        }
    }

    pub fn private(owned_state: OwnedIface, req: Req) -> Self {
        AssignIface {
            owned_state,
            public: false,
            required: req.is_required(),
            multiple: req.is_multiple(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD, tags = order)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub enum OwnedIface {
    #[strict_type(dumb)]
    Any,
    Rights,
    Amount,
    AnyData,
    AnyAttach,
    Data(SemId),
}

pub type ArgMap = TinyOrdMap<FieldName, Occurrences>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Display, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD, into_u8, try_from_u8, tags = repr)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
#[display(lowercase)]
#[repr(u8)]
pub enum Modifier {
    #[default]
    Final = 0,
    Abstract = 1,
    Override = 2,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct GenesisIface {
    pub modifier: Modifier,
    pub metadata: Option<SemId>,
    pub globals: ArgMap,
    pub assignments: ArgMap,
    pub valencies: TinyOrdSet<FieldName>,
    pub errors: TinyOrdSet<u8>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct ExtensionIface {
    pub modifier: Modifier,
    /// Defines whence schema may omit providing this operation.
    pub optional: bool,
    pub metadata: Option<SemId>,
    pub globals: ArgMap,
    pub assignments: ArgMap,
    pub redeems: TinyOrdSet<FieldName>,
    pub valencies: TinyOrdSet<FieldName>,
    pub errors: TinyOrdSet<u8>,
    pub default_assignment: Option<FieldName>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct TransitionIface {
    pub modifier: Modifier,
    /// Defines whence schema may omit providing this operation.
    pub optional: bool,
    pub metadata: Option<SemId>,
    pub globals: ArgMap,
    pub inputs: ArgMap,
    pub assignments: ArgMap,
    pub valencies: TinyOrdSet<FieldName>,
    pub errors: TinyOrdSet<u8>,
    pub default_assignment: Option<FieldName>,
}

/// Interface definition.
#[derive(Clone, Eq, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_STD)]
#[derive(CommitEncode)]
#[commit_encode(strategy = strict, id = IfaceId)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct Iface {
    pub version: VerNo,
    pub name: TypeName,
    pub inherits: TinyOrdSet<IfaceId>,
    pub global_state: TinyOrdMap<FieldName, GlobalIface>,
    pub assignments: TinyOrdMap<FieldName, AssignIface>,
    pub valencies: TinyOrdMap<FieldName, ValencyIface>,
    pub genesis: GenesisIface,
    pub transitions: TinyOrdMap<FieldName, TransitionIface>,
    pub extensions: TinyOrdMap<FieldName, ExtensionIface>,
    pub default_operation: Option<FieldName>,
    pub errors: TinyOrdMap<Variant, TinyString>,
    pub types: Types,
}

impl PartialEq for Iface {
    fn eq(&self, other: &Self) -> bool { self.iface_id() == other.iface_id() }
}

impl Ord for Iface {
    fn cmp(&self, other: &Self) -> Ordering { self.iface_id().cmp(&other.iface_id()) }
}

impl PartialOrd for Iface {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl StrictSerialize for Iface {}
impl StrictDeserialize for Iface {}

impl Iface {
    #[inline]
    pub fn iface_id(&self) -> IfaceId { self.commit_id() }

    pub fn display<'a>(
        &'a self,
        externals: HashMap<IfaceId, &'a TypeName>,
        sys: &'a SymbolicSys,
    ) -> IfaceDisplay<'a> {
        IfaceDisplay::new(self, externals, sys)
    }

    pub fn check(&self) -> Result<(), Vec<IfaceInconsistency>> {
        let proc_globals = |op_name: &OpName,
                            globals: &ArgMap,
                            errors: &mut Vec<IfaceInconsistency>| {
            for (name, occ) in globals {
                if let Some(g) = self.global_state.get(name) {
                    if occ.min_value() > 1 && !g.multiple {
                        errors.push(IfaceInconsistency::MultipleGlobal(
                            op_name.clone(),
                            name.clone(),
                        ));
                    }
                } else {
                    errors.push(IfaceInconsistency::UnknownGlobal(op_name.clone(), name.clone()));
                }
            }
        };
        let proc_assignments =
            |op_name: &OpName, assignments: &ArgMap, errors: &mut Vec<IfaceInconsistency>| {
                for (name, occ) in assignments {
                    if let Some(a) = self.assignments.get(name) {
                        if occ.min_value() > 1 && !a.multiple {
                            errors.push(IfaceInconsistency::MultipleAssignment(
                                op_name.clone(),
                                name.clone(),
                            ));
                        }
                    } else {
                        errors.push(IfaceInconsistency::UnknownAssignment(
                            op_name.clone(),
                            name.clone(),
                        ));
                    }
                }
            };
        let proc_valencies = |op_name: &OpName,
                              valencies: &TinyOrdSet<FieldName>,
                              errors: &mut Vec<IfaceInconsistency>| {
            for name in valencies {
                if self.valencies.get(name).is_none() {
                    errors.push(IfaceInconsistency::UnknownValency(op_name.clone(), name.clone()));
                }
            }
        };
        let proc_errors =
            |op_name: &OpName, errs: &TinyOrdSet<u8>, errors: &mut Vec<IfaceInconsistency>| {
                for tag in errs {
                    if self.errors.keys().all(|v| v.tag != *tag) {
                        errors.push(IfaceInconsistency::UnknownErrorTag(op_name.clone(), *tag));
                    }
                }
            };

        let mut errors = vec![];

        proc_globals(&OpName::Genesis, &self.genesis.globals, &mut errors);
        proc_assignments(&OpName::Genesis, &self.genesis.assignments, &mut errors);
        proc_valencies(&OpName::Genesis, &self.genesis.valencies, &mut errors);
        proc_errors(&OpName::Genesis, &self.genesis.errors, &mut errors);

        for (name, t) in &self.transitions {
            let op_name = OpName::Transition(name.clone());
            proc_globals(&op_name, &t.globals, &mut errors);
            proc_assignments(&op_name, &t.assignments, &mut errors);
            proc_valencies(&op_name, &t.valencies, &mut errors);
            proc_errors(&op_name, &t.errors, &mut errors);

            for (name, occ) in &t.inputs {
                if let Some(a) = self.assignments.get(name) {
                    if occ.min_value() > 1 && !a.multiple {
                        errors.push(IfaceInconsistency::MultipleInputs(
                            op_name.clone(),
                            name.clone(),
                        ));
                    }
                } else {
                    errors.push(IfaceInconsistency::UnknownInput(op_name.clone(), name.clone()));
                }
            }
            if let Some(ref name) = t.default_assignment {
                if t.assignments.get(name).is_none() {
                    errors
                        .push(IfaceInconsistency::UnknownDefaultAssignment(op_name, name.clone()));
                }
            }
        }

        for (name, e) in &self.extensions {
            let op_name = OpName::Extension(name.clone());
            proc_globals(&op_name, &e.globals, &mut errors);
            proc_assignments(&op_name, &e.assignments, &mut errors);
            proc_valencies(&op_name, &e.valencies, &mut errors);
            proc_errors(&op_name, &e.errors, &mut errors);

            for name in &e.redeems {
                if self.valencies.get(name).is_none() {
                    errors.push(IfaceInconsistency::UnknownRedeem(op_name.clone(), name.clone()));
                }
            }
            if let Some(ref name) = e.default_assignment {
                if e.assignments.get(name).is_none() {
                    errors
                        .push(IfaceInconsistency::UnknownDefaultAssignment(op_name, name.clone()));
                }
            }
        }

        for name in self.transitions.keys() {
            if self.extensions.contains_key(name) {
                errors.push(IfaceInconsistency::RepeatedOperationName(name.clone()));
            }
        }

        if let Some(ref name) = self.default_operation {
            if self.transitions.get(name).is_none() && self.extensions.get(name).is_none() {
                errors.push(IfaceInconsistency::UnknownDefaultOp(name.clone()));
            }
        }

        for (name, g) in &self.global_state {
            if g.required && self.genesis.globals.get(name).is_none() {
                errors.push(IfaceInconsistency::RequiredGlobalAbsent(name.clone()));
            }
        }
        for (name, a) in &self.assignments {
            if a.required && self.genesis.assignments.get(name).is_none() {
                errors.push(IfaceInconsistency::RequiredAssignmentAbsent(name.clone()));
            }
        }
        for (name, v) in &self.valencies {
            if v.required && self.genesis.valencies.get(name).is_none() {
                errors.push(IfaceInconsistency::RequiredValencyAbsent(name.clone()));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    // TODO: Implement checking interface inheritance.
    /*
    pub fn check_inheritance<'a>(&self, ifaces: impl IntoIterator<Item = (&'a IfaceId, &'a Iface)>) -> Result<(), Vec<InheritanceError>> {
        // check for the depth
    }
     */

    // TODO: Implement checking types against presence in a type system.
    /*
    pub fn check_types(&self, sys: &SymbolicSys) -> Result<(), Vec<IfaceTypeError>> {
        for g in self.global_state.values() {
            if let Some(id) = g.sem_id {

            }
        }
    }
     */
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display)]
pub enum OpName {
    #[display("genesis")]
    Genesis,
    #[display("transition '{0}'")]
    Transition(FieldName),
    #[display("extension '{0}'")]
    Extension(FieldName),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, Error)]
#[display(doc_comments)]
pub enum IfaceInconsistency {
    /// unknown global state '{1}' referenced from {0}.
    UnknownGlobal(OpName, FieldName),
    /// unknown valency '{1}' referenced from {0}.
    UnknownValency(OpName, FieldName),
    /// unknown input '{1}' referenced from {0}.
    UnknownRedeem(OpName, FieldName),
    /// unknown assignment '{1}' referenced from {0}.
    UnknownAssignment(OpName, FieldName),
    /// unknown input '{1}' referenced from {0}.
    UnknownInput(OpName, FieldName),
    /// unknown error tag '{1}' referenced from {0}.
    UnknownErrorTag(OpName, u8),
    /// unknown default assignment '{1}' referenced from {0}.
    UnknownDefaultAssignment(OpName, FieldName),
    /// unknown default operation '{0}'.
    UnknownDefaultOp(FieldName),
    /// global state '{1}' must have a unique single value, but operation {0}
    /// defines multiple global state of this type.
    MultipleGlobal(OpName, FieldName),
    /// assignment '{1}' must be unique, but operation {0} defines multiple
    /// assignments of this type.
    MultipleAssignment(OpName, FieldName),
    /// assignment '{1}' is unique, but operation {0} defines multiple inputs of
    /// this type, which is not possible.
    MultipleInputs(OpName, FieldName),
    /// operation name '{0}' is used by both state transition and extension.
    RepeatedOperationName(FieldName),
    /// global state '{0}' is required, but genesis doesn't define it.
    RequiredGlobalAbsent(FieldName),
    /// assignment '{0}' is required, but genesis doesn't define it.
    RequiredAssignmentAbsent(FieldName),
    /// valency '{0}' is required, but genesis doesn't define it.
    RequiredValencyAbsent(FieldName),
}
