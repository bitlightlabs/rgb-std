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

#![allow(unused_braces)] // caused by rustc unable to understand strict_dumb

use std::fmt::{self, Debug, Formatter};
use std::str::FromStr;

use amplify::ascii::AsciiString;
use amplify::confinement::{Confined, NonEmptyString, NonEmptyVec, SmallOrdSet, SmallString, U8};
use invoice::Precision;
use strict_encoding::stl::{AlphaCapsNum, AsciiPrintable};
use strict_encoding::{
    InvalidIdent, StrictDeserialize, StrictDumb, StrictEncode, StrictSerialize, StrictType,
    TypedWrite,
};
use strict_types::StrictVal;

use super::{MediaType, ProofOfReserves, LIB_NAME_RGB_CONTRACT};

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct BurnMeta {
    pub burn_proofs: SmallOrdSet<ProofOfReserves>,
}
impl StrictSerialize for BurnMeta {}
impl StrictDeserialize for BurnMeta {}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Default)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct IssueMeta {
    pub reserves: SmallOrdSet<ProofOfReserves>,
}
impl StrictSerialize for IssueMeta {}
impl StrictDeserialize for IssueMeta {}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, From)]
#[wrapper(Deref, Display)]
#[derive(StrictDumb, StrictType, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT, dumb = { Ticker::from("DUMB") })]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Ticker(Confined<AsciiString, 1, 8>);
impl StrictEncode for Ticker {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        let iter = self
            .0
            .as_bytes()
            .iter()
            .map(|c| AlphaCapsNum::try_from(*c).unwrap());
        writer.write_newtype::<Self>(&NonEmptyVec::<AlphaCapsNum, 8>::try_from_iter(iter).unwrap())
    }
}
impl StrictSerialize for Ticker {}
impl StrictDeserialize for Ticker {}

impl AsRef<str> for Ticker {
    #[inline]
    fn as_ref(&self) -> &str { self.0.as_str() }
}

// TODO: Ensure all constructors filter invalid characters
impl FromStr for Ticker {
    type Err = InvalidIdent;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = AsciiString::from_ascii(s.as_bytes())?;
        Self::try_from(s)
    }
}

impl From<&'static str> for Ticker {
    fn from(s: &'static str) -> Self { Self::from_str(s).expect("invalid ticker name") }
}

impl TryFrom<String> for Ticker {
    type Error = InvalidIdent;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let s = AsciiString::from_ascii(s.as_bytes())?;
        Self::try_from(s)
    }
}

impl TryFrom<AsciiString> for Ticker {
    type Error = InvalidIdent;

    fn try_from(ascii: AsciiString) -> Result<Self, InvalidIdent> {
        if ascii.is_empty() {
            return Err(InvalidIdent::Empty);
        }
        if let Some(ch) = ascii
            .as_slice()
            .iter()
            .copied()
            .find(|ch| AlphaCapsNum::try_from(ch.as_byte()).is_err())
        {
            return Err(InvalidIdent::InvalidChar(ascii, ch));
        }
        let s = Confined::try_from(ascii)?;
        Ok(Self(s))
    }
}

impl Debug for Ticker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Ticker").field(&self.as_str()).finish()
    }
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, From)]
#[wrapper(Deref, Display)]
#[derive(StrictType, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Name(Confined<AsciiString, 1, 40>);
impl StrictEncode for Name {
    fn strict_encode<W: TypedWrite>(&self, writer: W) -> std::io::Result<W> {
        let iter = self
            .0
            .as_bytes()
            .iter()
            .map(|c| AsciiPrintable::try_from(*c).unwrap());
        writer
            .write_newtype::<Self>(&NonEmptyVec::<AsciiPrintable, 40>::try_from_iter(iter).unwrap())
    }
}
impl StrictSerialize for Name {}
impl StrictDeserialize for Name {}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str { self.0.as_str() }
}

impl Name {
    pub fn from_strict_val_unchecked(value: &StrictVal) -> Self {
        Name::from_str(&value.unwrap_string()).unwrap()
    }
}

impl StrictDumb for Name {
    fn strict_dumb() -> Self { Name::from("Dumb contract name") }
}

// TODO: Ensure all constructors filter invalid characters
impl FromStr for Name {
    type Err = InvalidIdent;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = AsciiString::from_ascii(s.as_bytes())?;
        Self::try_from(s)
    }
}

impl TryFrom<AsciiString> for Name {
    type Error = InvalidIdent;

    fn try_from(ascii: AsciiString) -> Result<Self, InvalidIdent> {
        if ascii.is_empty() {
            return Err(InvalidIdent::Empty);
        }
        if let Some(ch) = ascii
            .as_slice()
            .iter()
            .copied()
            .find(|ch| AsciiPrintable::try_from(ch.as_byte()).is_err())
        {
            return Err(InvalidIdent::InvalidChar(ascii, ch));
        }
        let s = Confined::try_from(ascii)?;
        Ok(Self(s))
    }
}

impl From<&'static str> for Name {
    fn from(s: &'static str) -> Self { Self::from_str(s).expect("invalid ticker name") }
}

impl TryFrom<String> for Name {
    type Error = InvalidIdent;

    fn try_from(name: String) -> Result<Self, InvalidIdent> {
        let name = AsciiString::from_ascii(name.as_bytes())?;
        Self::try_from(name)
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ContractName").field(&self.as_str()).finish()
    }
}

#[derive(Wrapper, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, From)]
#[wrapper(Deref, Display)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct Details(NonEmptyString<U8>);
impl StrictSerialize for Details {}
impl StrictDeserialize for Details {}

impl AsRef<str> for Details {
    #[inline]
    fn as_ref(&self) -> &str { self.0.as_str() }
}

impl Details {
    pub fn from_strict_val_unchecked(value: &StrictVal) -> Self {
        Details::from_str(&value.unwrap_string()).unwrap()
    }
}

impl StrictDumb for Details {
    fn strict_dumb() -> Self {
        Self(Confined::try_from(s!("Dumb long description which is stupid and so on...")).unwrap())
    }
}

impl FromStr for Details {
    type Err = InvalidIdent;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = Confined::try_from_iter(s.chars())?;
        Ok(Self(s))
    }
}

impl From<&'static str> for Details {
    fn from(s: &'static str) -> Self { Self::from_str(s).expect("invalid ticker name") }
}

impl TryFrom<String> for Details {
    type Error = InvalidIdent;

    fn try_from(name: String) -> Result<Self, InvalidIdent> {
        let s = Confined::try_from(name)?;
        Ok(Self(s))
    }
}

impl Debug for Details {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ContractDetails")
            .field(&self.as_str())
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct AssetSpec {
    pub ticker: Ticker,
    pub name: Name,
    pub details: Option<Details>,
    pub precision: Precision,
}
impl StrictSerialize for AssetSpec {}
impl StrictDeserialize for AssetSpec {}

impl AssetSpec {
    pub fn new(ticker: &'static str, name: &'static str, precision: Precision) -> AssetSpec {
        AssetSpec {
            ticker: Ticker::from(ticker),
            name: Name::from(name),
            details: None,
            precision,
        }
    }

    pub fn with(
        ticker: &str,
        name: &str,
        precision: Precision,
        details: Option<&str>,
    ) -> Result<AssetSpec, InvalidIdent> {
        Ok(AssetSpec {
            ticker: Ticker::try_from(ticker.to_owned())?,
            name: Name::try_from(name.to_owned())?,
            details: details.map(Details::from_str).transpose()?,
            precision,
        })
    }

    pub fn from_strict_val_unchecked(value: &StrictVal) -> Self {
        let ticker = value.unwrap_struct("ticker").unwrap_string();
        let name = value.unwrap_struct("name").unwrap_string();
        let details = value
            .unwrap_struct("details")
            .unwrap_option()
            .map(StrictVal::unwrap_string);
        let precision = value.unwrap_struct("precision").unwrap_enum();
        Self {
            ticker: Ticker::from_str(&ticker).expect("invalid asset ticker"),
            name: Name::from_str(&name).expect("invalid asset name"),
            details: details
                .as_deref()
                .map(Details::from_str)
                .transpose()
                .expect("invalid asset details"),
            precision,
        }
    }

    pub fn ticker(&self) -> &str { self.ticker.as_str() }

    pub fn name(&self) -> &str { self.name.as_str() }

    pub fn details(&self) -> Option<&str> { self.details.as_ref().map(|d| d.as_str()) }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug, Display, Default)]
#[display(inner)]
#[derive(StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", transparent)
)]
pub struct RicardianContract(SmallString);
impl StrictSerialize for RicardianContract {}
impl StrictDeserialize for RicardianContract {}

impl AsRef<str> for RicardianContract {
    #[inline]
    fn as_ref(&self) -> &str { self.0.as_str() }
}

impl FromStr for RicardianContract {
    type Err = InvalidIdent;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = Confined::try_from_iter(s.chars())?;
        Ok(Self(s))
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictType, StrictDumb, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct Attachment {
    #[strict_type(rename = "type")]
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub ty: MediaType,
    pub digest: [u8; 32],
}
impl StrictSerialize for Attachment {}
impl StrictDeserialize for Attachment {}

impl Attachment {
    pub fn from_strict_val_unchecked(value: &StrictVal) -> Self {
        let ty = MediaType::from_strict_val_unchecked(value.unwrap_struct("type"));
        let digest = value
            .unwrap_struct("digest")
            .unwrap_bytes()
            .try_into()
            .expect("invalid digest");
        Self { ty, digest }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictDumb, StrictType, StrictEncode, StrictDecode)]
#[strict_type(lib = LIB_NAME_RGB_CONTRACT)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "camelCase")
)]
pub struct AssetTerms {
    pub text: RicardianContract,
    pub media: Option<Attachment>,
}
impl StrictSerialize for AssetTerms {}
impl StrictDeserialize for AssetTerms {}

impl AssetTerms {
    pub fn from_strict_val_unchecked(value: &StrictVal) -> Self {
        let text = RicardianContract::from_str(&value.unwrap_struct("text").unwrap_string())
            .expect("invalid text");
        let media = value
            .unwrap_struct("media")
            .unwrap_option()
            .map(Attachment::from_strict_val_unchecked);
        Self { text, media }
    }
}
