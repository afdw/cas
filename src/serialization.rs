use crate::{data::*, Value};
use indexmap::map::IndexMap;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

type JsonValue = serde_json::Value;
type JsonNumber = serde_json::Number;

fn serialize_one<F: FnMut(Value) -> JsonValue>(value: Value, mut f: F) -> JsonValue {
    if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        json!({
            "type": "Hold",
            "inner": f(value_inner.inner.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<ReleaseValueInner>() {
        json!({
            "type": "Release",
            "inner": f(value_inner.inner.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        json!({
            "type": "Assignment",
            "source": f(value_inner.source.clone()),
            "target": f(value_inner.target.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<DereferenceValueInner>() {
        json!({
            "type": "Dereference",
            "inner": f(value_inner.inner.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        json!({
            "type": "ExecutableSequence",
            "inner": JsonValue::Array(value_inner.inner.iter().cloned().map(&mut f).collect()),
        })
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        json!({
            "type": "ExecutableFunction",
            "arguments": f(value_inner.arguments.clone()),
            "body": f(value_inner.body.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        json!({
            "type": "FunctionApplication",
            "function": f(value_inner.function.clone()),
            "arguments": f(value_inner.arguments.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        json!({
            "type": "IntrinsicCall",
            "intrinsic": f(value_inner.intrinsic.clone()),
            "arguments": f(value_inner.arguments.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<TupleValueInner>() {
        json!({
            "type": "Tuple",
            "inner": JsonValue::Array(value_inner.inner.iter().cloned().map(&mut f).collect()),
        })
    } else if let Some(_) = value.try_downcast::<NullValueInner>() {
        json!({
            "type": "Null",
        })
    } else if let Some(value_inner) = value.try_downcast::<SymbolValueInner>() {
        json!({
            "type": "Symbol",
            "name": JsonValue::String(value_inner.name.clone()),
        })
    } else if let Some(value_inner) = value.try_downcast::<FloatingPointNumberValueInner>() {
        json!({
            "type": "FloatingPointNumber",
            "inner": JsonValue::Number(JsonNumber::from_f64(value_inner.inner).unwrap()),
        })
    } else {
        unreachable!()
    }
}

fn deserialize_one<F: FnMut(JsonValue) -> Value>(entry: &JsonValue, mut f: F) -> Value {
    match entry["type"].as_str().unwrap() {
        "Hold" => Value::new(HoldValueInner {
            inner: f(entry["inner"].clone()),
        }),
        "Release" => Value::new(ReleaseValueInner {
            inner: f(entry["inner"].clone()),
        }),
        "Assignment" => Value::new(AssignmentValueInner {
            source: f(entry["source"].clone()),
            target: f(entry["target"].clone()),
        }),
        "Dereference" => Value::new(DereferenceValueInner {
            inner: f(entry["inner"].clone()),
        }),
        "ExecutableSequence" => Value::new(ExecutableSequenceValueInner {
            inner: entry["inner"].clone().as_array().unwrap().iter().cloned().map(&mut f).collect(),
        }),
        "ExecutableFunction" => Value::new(ExecutableFunctionValueInner {
            arguments: f(entry["arguments"].clone()),
            body: f(entry["body"].clone()),
        }),
        "FunctionApplication" => Value::new(FunctionApplicationValueInner {
            function: f(entry["function"].clone()),
            arguments: f(entry["arguments"].clone()),
        }),
        "IntrinsicCall" => Value::new(IntrinsicCallValueInner {
            intrinsic: f(entry["intrinsic"].clone()),
            arguments: f(entry["arguments"].clone()),
        }),
        "Tuple" => Value::new(TupleValueInner {
            inner: entry["inner"].clone().as_array().unwrap().iter().cloned().map(&mut f).collect(),
        }),
        "Null" => Value::new(NullValueInner),
        "Symbol" => Value::new(SymbolValueInner {
            name: entry["name"].as_str().unwrap().to_owned(),
        }),
        "FloatingPointNumber" => Value::new(FloatingPointNumberValueInner {
            inner: entry["inner"].as_f64().unwrap(),
        }),
        _ => unreachable!(),
    }
}

pub fn serialize_readable(value: Value) -> String {
    fn f(value: Value) -> JsonValue {
        serialize_one(value, f)
    };
    serde_json::to_string_pretty(&f(value)).unwrap()
}

pub struct SerializationStorage {
    known_ids: IndexMap<Value, Uuid>,
    known_value: HashMap<Uuid, Value>,
}

impl SerializationStorage {
    pub(crate) fn new() -> Self {
        SerializationStorage {
            known_ids: IndexMap::new(),
            known_value: HashMap::new(),
        }
    }
}

fn serialize_id(id: Uuid) -> JsonValue {
    JsonValue::String(format!("{:X}", id.to_simple()))
}

fn deserialize_id(id: &JsonValue) -> Uuid {
    Uuid::parse_str(id.as_str().unwrap()).unwrap()
}

pub fn serialize(serialization_storage: &mut SerializationStorage, input_value: Value) -> String {
    let mut done = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(input_value.clone());
    let mut unordered = HashMap::new();
    while !queue.is_empty() {
        let current = queue.pop_front().unwrap();
        done.insert(current.clone());
        let mut f = |value| {
            if !done.contains(&value) {
                queue.push_back(value.clone());
            }
            serialize_id(*serialization_storage.known_ids.entry(value).or_insert_with(Uuid::new_v4))
        };
        let mut entry = serialize_one(current.clone(), &mut f);
        entry["id"] = f(current.clone());
        unordered.insert(current, entry);
    }
    let mut ordered = unordered.into_iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(value, _)| serialization_storage.known_ids.get_index_of(value).unwrap());
    serde_json::to_string_pretty(&json!({
        "id": serialize_id(serialization_storage.known_ids[&input_value]),
        "values": JsonValue::Array(ordered.into_iter().map(|(_, entry)| entry).collect()),
    }))
    .unwrap()
}

pub fn deserialize(mut serialization_storage: &mut SerializationStorage, input_str: &str) -> Value {
    let parsed: JsonValue = serde_json::from_str(input_str).unwrap();
    let mut deserialized = HashMap::new();
    fn f(serialization_storage: &mut SerializationStorage, deserialized: &mut HashMap<Uuid, Value>, entry: &JsonValue) -> Value {
        let id = deserialize_id(&entry["id"]);
        if let Some(value) = serialization_storage.known_value.get(&id) {
            value.clone()
        } else {
            let value = deserialize_one(entry, |entry| f(serialization_storage, deserialized, &entry));
            if serialization_storage.known_ids.contains_key(&value) {
                serialization_storage.known_ids.shift_remove(&value);
            }
            serialization_storage.known_ids.insert(value.clone(), id);
            serialization_storage.known_value.insert(id, value.clone());
            value
        }
    }
    f(
        &mut serialization_storage,
        &mut deserialized,
        parsed["values"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| deserialize_id(&entry["id"]) == deserialize_id(&parsed["id"]))
            .unwrap(),
    )
}
