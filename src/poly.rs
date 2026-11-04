//! Polynomial files emitted by external NFS instruments.
//!
//! Version `instrument-polynomial/2` names the closed lists (version 1, degree at most 6, stays on disk). The file
//! identifies its format. It does not identify which program wrote it, so
//! the source stays `unspecified` unless the caller sets one.

use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt;

pub const V1: &str = "instrument-polynomial/2";
pub const MAX_DEGREE: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Cado,
    MsieveFb,
}

impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cado => "cado",
            Self::MsieveFb => "msieve-fb",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    CadoNfs,
    Msieve,
    Yafu,
    Unspecified,
}

impl Source {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CadoNfs => "cado-nfs",
            Self::Msieve => "msieve",
            Self::Yafu => "yafu",
            Self::Unspecified => "unspecified",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstrumentPolynomial {
    pub format: Format,
    pub n: BigUint,
    pub skew: Option<String>,
    pub algebraic: Vec<BigInt>,
    pub rational: Vec<BigInt>,
}

impl InstrumentPolynomial {
    pub fn degree(&self) -> u32 {
        (self.algebraic.len() - 1) as u32
    }

    pub fn n_digits(&self) -> usize {
        self.n.to_string().len()
    }

    pub fn record(&self, source: Source) -> Value {
        let mut object = Map::new();
        object.insert("schema_version".into(), Value::String(V1.into()));
        object.insert("format".into(), Value::String(self.format.as_str().into()));
        object.insert("source".into(), Value::String(source.as_str().into()));
        object.insert("n".into(), Value::String(self.n.to_string()));
        if let Some(skew) = &self.skew {
            object.insert("skew".into(), Value::String(skew.clone()));
        }
        object.insert("algebraic".into(), coeff_array(&self.algebraic));
        object.insert("rational".into(), coeff_array(&self.rational));
        object.insert("degree".into(), Value::from(self.degree()));
        Value::Object(object)
    }
}

fn coeff_array(coeffs: &[BigInt]) -> Value {
    Value::Array(
        coeffs
            .iter()
            .map(|coeff| Value::String(coeff.to_string()))
            .collect(),
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    NotInstrument,
    Invalid(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInstrument => write!(f, "not an instrument polynomial file"),
            Self::Invalid(message) => write!(f, "{message}"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Algebraic,
    Rational,
}

struct Draft {
    format: Option<Format>,
    n: Option<BigUint>,
    skew: Option<String>,
    algebraic: BTreeMap<usize, BigInt>,
    rational: BTreeMap<usize, BigInt>,
    saw_key: bool,
}

pub fn parse_instrument(text: &str) -> Result<InstrumentPolynomial, Error> {
    let mut draft = Draft {
        format: None,
        n: None,
        skew: None,
        algebraic: BTreeMap::new(),
        rational: BTreeMap::new(),
        saw_key: false,
    };
    let mut skipped = false;
    for (number, raw) in text.lines().enumerate() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = split_key(line) else {
            if draft.saw_key {
                return Err(Error::Invalid(format!(
                    "line {}: unrecognized content",
                    number + 1
                )));
            }
            skipped = true;
            continue;
        };
        let Some(family) = family_of(&key) else {
            if draft.saw_key {
                return Err(Error::Invalid(format!(
                    "line {}: unrecognized key {key}",
                    number + 1
                )));
            }
            skipped = true;
            continue;
        };
        if skipped {
            return Err(Error::Invalid(
                "unrecognized lines appear before the polynomial keys".into(),
            ));
        }
        draft.saw_key = true;
        if let Some(existing) = draft.format {
            if existing != family {
                return Err(Error::Invalid(
                    "file mixes cado and msieve-fb polynomial keys".into(),
                ));
            }
        } else {
            draft.format = Some(family);
        }
        apply_key(&mut draft, &key, value.trim(), number + 1)?;
    }
    let Some(format) = draft.format else {
        return Err(Error::NotInstrument);
    };
    let n = draft
        .n
        .ok_or_else(|| Error::Invalid("instrument polynomial is missing n".into()))?;
    let algebraic = dense(&draft.algebraic, "algebraic")?;
    let rational = dense(&draft.rational, "rational")?;
    let poly = InstrumentPolynomial {
        format,
        n,
        skew: draft.skew,
        algebraic,
        rational,
    };
    if poly.degree() < 1 || poly.degree() > MAX_DEGREE {
        return Err(Error::Invalid(format!(
            "algebraic degree {} is outside 1..={MAX_DEGREE} for {V1}",
            poly.degree()
        )));
    }
    Ok(poly)
}

fn strip_comment(line: &str) -> &str {
    line.split('#').next().unwrap_or("")
}

fn split_key(line: &str) -> Option<(String, &str)> {
    if let Some((key, value)) = line.split_once('=') {
        let key = key.trim();
        if key.starts_with("poly") {
            return Some((key.to_string(), value));
        }
    }
    if let Some((key, value)) = line.split_once(':') {
        return Some((key.trim().to_string(), value));
    }
    let mut parts = line.split_whitespace();
    let key = parts.next()?;
    let value = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((key.to_string(), value))
}

fn family_of(key: &str) -> Option<Format> {
    if key == "n" || key == "skew" || key.starts_with("poly") {
        return Some(Format::Cado);
    }
    if indexed(key, &['c', 'Y', 'X']) {
        return Some(Format::Cado);
    }
    if key == "N" || key == "SKEW" || indexed(key, &['A', 'R']) {
        return Some(Format::MsieveFb);
    }
    None
}

fn indexed(key: &str, prefixes: &[char]) -> bool {
    let mut chars = key.chars();
    let Some(prefix) = chars.next() else {
        return false;
    };
    if !prefixes.contains(&prefix) {
        return false;
    }
    let rest: String = chars.collect();
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

fn apply_key(draft: &mut Draft, key: &str, value: &str, line: usize) -> Result<(), Error> {
    if value.is_empty() {
        return Err(Error::Invalid(format!("line {line}: {key} has no value")));
    }
    match key {
        "n" | "N" => {
            let n = parse_n(value).map_err(|err| Error::Invalid(format!("line {line}: {err}")))?;
            if draft.n.as_ref().is_some_and(|existing| existing != &n) {
                return Err(Error::Invalid(format!(
                    "line {line}: n disagrees with an earlier value"
                )));
            }
            draft.n = Some(n);
        }
        "skew" | "SKEW" => {
            if !is_skew(value) {
                return Err(Error::Invalid(format!(
                    "line {line}: skew is not a decimal"
                )));
            }
            if draft
                .skew
                .as_ref()
                .is_some_and(|existing| existing != value)
            {
                return Err(Error::Invalid(format!(
                    "line {line}: skew disagrees with an earlier value"
                )));
            }
            draft.skew = Some(value.to_string());
        }
        "poly0" => set_poly(draft, Side::Rational, value, line)?,
        "poly1" => set_poly(draft, Side::Algebraic, value, line)?,
        _ if key.starts_with("poly") => {
            return Err(Error::Invalid(format!(
                "line {line}: {V1} accepts poly0 and poly1 only"
            )));
        }
        _ => {
            let (side, index) = coeff_slot(key)
                .ok_or_else(|| Error::Invalid(format!("line {line}: unrecognized key {key}")))?;
            let coeff =
                parse_coeff(value).map_err(|err| Error::Invalid(format!("line {line}: {err}")))?;
            set_coeff(draft, side, index, coeff, line)?;
        }
    }
    Ok(())
}

fn coeff_slot(key: &str) -> Option<(Side, usize)> {
    let (prefix, rest) = key.split_at(1);
    let index: usize = rest.parse().ok()?;
    let side = match prefix {
        "c" | "X" | "A" => Side::Algebraic,
        "Y" | "R" => Side::Rational,
        _ => return None,
    };
    Some((side, index))
}

fn set_poly(draft: &mut Draft, side: Side, value: &str, line: usize) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(Error::Invalid(format!("line {line}: empty polynomial")));
    }
    for (index, part) in value.split(',').enumerate() {
        let coeff = parse_coeff(part.trim())
            .map_err(|err| Error::Invalid(format!("line {line}: {err}")))?;
        set_coeff(draft, side, index, coeff, line)?;
    }
    Ok(())
}

fn set_coeff(
    draft: &mut Draft,
    side: Side,
    index: usize,
    coeff: BigInt,
    line: usize,
) -> Result<(), Error> {
    if index > MAX_DEGREE as usize {
        return Err(Error::Invalid(format!(
            "line {line}: coefficient index {index} is above {MAX_DEGREE} for {V1}"
        )));
    }
    let map = match side {
        Side::Algebraic => &mut draft.algebraic,
        Side::Rational => &mut draft.rational,
    };
    if let Some(existing) = map.get(&index) {
        if existing != &coeff {
            return Err(Error::Invalid(format!(
                "line {line}: coefficient {index} disagrees with an earlier value"
            )));
        }
    } else {
        map.insert(index, coeff);
    }
    Ok(())
}

fn dense(map: &BTreeMap<usize, BigInt>, name: &str) -> Result<Vec<BigInt>, Error> {
    if map.is_empty() {
        return Err(Error::Invalid(format!(
            "instrument polynomial is missing {name} coefficients"
        )));
    }
    let max = *map.keys().next_back().expect("map is non-empty");
    let mut coeffs = vec![BigInt::zero(); max + 1];
    for (index, coeff) in map {
        coeffs[*index] = coeff.clone();
    }
    while coeffs.last().is_some_and(Zero::is_zero) {
        coeffs.pop();
    }
    if coeffs.is_empty() {
        return Err(Error::Invalid(format!(
            "instrument polynomial is missing {name} coefficients"
        )));
    }
    Ok(coeffs)
}

fn parse_n(text: &str) -> Result<BigUint, String> {
    if !text.chars().all(|c| c.is_ascii_digit()) || text.starts_with('0') {
        return Err(format!("n {text} is not a positive decimal integer"));
    }
    BigUint::parse_bytes(text.as_bytes(), 10).ok_or_else(|| format!("n {text} is not an integer"))
}

fn parse_coeff(text: &str) -> Result<BigInt, String> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty()
        || !digits.chars().all(|c| c.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
    {
        return Err(format!("coefficient {text} is not a decimal integer"));
    }
    BigInt::parse_bytes(text.as_bytes(), 10)
        .ok_or_else(|| format!("coefficient {text} is not an integer"))
}

fn is_skew(text: &str) -> bool {
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (unsigned, None),
    };
    let (whole, fraction) = match mantissa.split_once('.') {
        Some((whole, fraction)) => {
            if fraction.is_empty() || !fraction.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            (whole, true)
        }
        None => (mantissa, false),
    };
    let _ = fraction;
    if whole != "0"
        && (whole.is_empty()
            || whole.starts_with('0')
            || !whole.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }
    if !whole.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let Some(exponent) = exponent else {
        return true;
    };
    let exponent = exponent
        .strip_prefix('+')
        .or_else(|| exponent.strip_prefix('-'))
        .unwrap_or(exponent);
    !exponent.is_empty() && exponent.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn schema() -> Value {
        serde_json::from_str(include_str!("../schemas/instrument-polynomial/2.json"))
            .expect("schema json")
    }

    fn enum_strings(schema: &Value, field: &str) -> Vec<String> {
        schema["properties"][field]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_string())
            .collect()
    }

    #[test]
    fn closed_lists_match_the_schema() {
        let schema = schema();
        assert_eq!(schema["$id"].as_str(), Some(V1));
        assert_eq!(
            enum_strings(&schema, "format"),
            vec![Format::Cado.as_str(), Format::MsieveFb.as_str()]
        );
        assert_eq!(
            enum_strings(&schema, "source"),
            vec![
                Source::CadoNfs.as_str(),
                Source::Msieve.as_str(),
                Source::Yafu.as_str(),
                Source::Unspecified.as_str()
            ]
        );
        assert_eq!(
            schema["properties"]["degree"]["maximum"].as_u64(),
            Some(MAX_DEGREE as u64)
        );
    }

    #[test]
    fn parses_cado_and_msieve_files_into_the_schema() {
        let schema = schema();
        let validator = jsonschema::validator_for(&schema).expect("schema compiles");
        for (text, format) in [
            (
                include_str!("../examples/instrument_cado.poly"),
                Format::Cado,
            ),
            (
                include_str!("../examples/instrument_msieve.fb"),
                Format::MsieveFb,
            ),
        ] {
            let poly = parse_instrument(text).expect("parse");
            assert_eq!(poly.format, format);
            assert_eq!(poly.degree(), 4);
            assert_eq!(poly.n_digits(), 7);
            let record = poly.record(Source::Unspecified);
            validator
                .validate(&record)
                .unwrap_or_else(|err| panic!("{err}"));
        }
    }

    #[test]
    fn parses_cado_poly_equals_lines() {
        let text = "n: 1000003\nskew: 1.5\npoly0=-2,1\npoly1=-1,0,0,0,1\n";
        let poly = parse_instrument(text).unwrap();
        assert_eq!(poly.format, Format::Cado);
        assert_eq!(poly.rational, vec![BigInt::from(-2), BigInt::from(1)]);
        assert_eq!(poly.degree(), 4);
    }

    #[test]
    fn leaves_the_toy_coeff_list_alone_and_rejects_a_score_key() {
        let toy = include_str!("../examples/demo.poly");
        assert_eq!(parse_instrument(toy), Err(Error::NotInstrument));
        let scored = "n: 1000003\nc0: -1\nc4: 1\nY0: -2\nY1: 1\nMurphyE: 1.2e-8\n";
        let err = parse_instrument(scored).unwrap_err();
        assert!(matches!(err, Error::Invalid(message) if message.contains("MurphyE")));
    }
}
