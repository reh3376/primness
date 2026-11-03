use crate::metrics::Event;
use jsonschema::Validator;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const V1: &str = "measurement-event/1";

/// Schemas loaded from disk. A new version is a new file under `schemas/`.
/// An event is collected only when its `schema_version` names one of these
/// documents and the JSON matches that document.
struct Compiled {
    validator: Validator,
    document: Value,
}

pub struct Registry {
    validators: HashMap<String, Compiled>,
}

impl Registry {
    pub fn load(dir: &Path) -> Result<Self, String> {
        let mut validators = HashMap::new();
        let mut files = Vec::new();
        walk(dir, &mut files)?;
        if files.is_empty() {
            return Err(format!("no JSON Schemas in {}", dir.display()));
        }
        for path in files {
            let text =
                fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))?;
            let document: Value =
                serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
            let id = document
                .get("$id")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("{} has no $id", path.display()))?
                .to_string();
            let validator =
                jsonschema::validator_for(&document).map_err(|err| format!("{id}: {err}"))?;
            validators.insert(
                id,
                Compiled {
                    validator,
                    document,
                },
            );
        }
        Ok(Self { validators })
    }

    pub fn from_env() -> Result<Self, String> {
        let dir = std::env::var("PRIMNESS_SCHEMAS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/schemas")));
        Self::load(&dir)
    }

    pub fn validate(&self, event: &Event) -> Result<(), String> {
        let compiled = self.validators.get(&event.schema_version).ok_or_else(|| {
            format!(
                "unknown schema_version {}. Known: {}",
                event.schema_version,
                self.known()
            )
        })?;
        let value = serde_json::to_value(event).map_err(|err| err.to_string())?;
        match compiled.validator.validate(&value) {
            Ok(()) => Ok(()),
            Err(err) => Err(format!("{}: {err}", event.schema_version)),
        }
    }

    pub fn documents(&self) -> Vec<(String, Value)> {
        self.validators
            .iter()
            .map(|(id, compiled)| (id.clone(), compiled.document.clone()))
            .collect()
    }

    fn known(&self) -> String {
        let mut ids: Vec<_> = self.validators.keys().cloned().collect();
        ids.sort();
        ids.join(", ")
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|err| format!("{}: {err}", dir.display()))?;
    for entry in entries {
        let path = entry.map_err(|err| err.to_string())?.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "json") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metrics::Event;

    fn sample(kind: &str) -> Event {
        Event {
            schema_version: V1.into(),
            ts_ms: 10,
            run_id: "run-a".into(),
            source: "d1-measure".into(),
            kind: kind.into(),
            scale: "toy".into(),
            instance: "demo.poly".into(),
            b: 32,
            evaluated: 10,
            smooth: 4,
            elapsed_ms: 2,
            rate_per_s: 5000.0,
            residues: "1100".into(),
        }
    }

    #[test]
    fn accepts_version_one_and_rejects_a_new_kind() {
        let registry = Registry::from_env().unwrap();
        registry.validate(&sample("sample")).unwrap();
        let err = registry.validate(&sample("progress")).unwrap_err();
        assert!(err.contains("measurement-event/1"), "{err}");
    }

    #[test]
    fn rejects_an_unknown_version() {
        let registry = Registry::from_env().unwrap();
        let mut event = sample("start");
        event.schema_version = "measurement-event/99".into();
        event.residues.clear();
        let err = registry.validate(&event).unwrap_err();
        assert!(err.contains("unknown schema_version"), "{err}");
    }

    #[test]
    fn current_state_contract_consts_satisfy_its_schema() {
        let registry = Registry::from_env().unwrap();
        let document = registry
            .documents()
            .into_iter()
            .find(|(id, _)| id == "current-state/1")
            .map(|(_, document)| document)
            .expect("current-state/1 is loaded");
        let instance = instance_from_consts(&document);
        let validator = jsonschema::validator_for(&document).expect("schema compiles");
        validator
            .validate(&instance)
            .unwrap_or_else(|err| panic!("{err}"));
    }

    fn instance_from_consts(schema: &Value) -> Value {
        let properties = schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("properties");
        let mut object = serde_json::Map::new();
        for (key, property) in properties {
            let constant = property
                .get("const")
                .unwrap_or_else(|| panic!("{key} has no const"));
            object.insert(key.clone(), constant.clone());
        }
        Value::Object(object)
    }
}
