use surrealdb::sql::{Array, Value};

pub fn array_response(response: Value) -> Value {
	match response {
		Value::Array(_) => response,
		_ => Value::Array(Array::from(response))
	}
}
