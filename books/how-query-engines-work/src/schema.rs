use arrow::datatypes::{FieldRef, Fields, Schema as ArrowSchema, SchemaRef as ArrowSchemaRef};
use std::sync::Arc;

pub type SchemaRef = Arc<Schema>;

#[derive(Clone, Debug, PartialEq)]
pub struct Schema {
    fields: Fields,
}

impl Schema {
    pub fn from_fields<T>(fields: impl IntoIterator<Item = T>) -> Self
    where
        T: Into<FieldRef>,
    {
        Self {
            fields: fields.into_iter().map(Into::into).collect(),
        }
    }

    /// Create a new [`Schema`] from an [`ArrowSchemaRef`] by cheaply cloning its fields.
    pub fn from_arrow(schema: ArrowSchemaRef) -> Self {
        Self {
            fields: schema.fields().clone(),
        }
    }

    pub fn to_arrow(&self) -> ArrowSchemaRef {
        Arc::new(ArrowSchema::new(self.fields.clone()))
    }

    /// Returns an immutable reference of the vector of [`Field`] instances.
    pub fn fields(&self) -> &Fields {
        &self.fields
    }

    /// Create a new [`Schema`] from the existing schema by selecting [`Field`]'s by index.
    pub fn project(&self, indices: impl IntoIterator<Item = usize>) -> Self {
        Self {
            fields: indices
                .into_iter()
                .map(|i| self.fields[i].clone())
                .collect(),
        }
    }

    pub fn projection_mask(&self, names: &[&str]) -> impl Iterator<Item = usize> {
        self.fields
            .into_iter()
            .enumerate()
            .filter(|(_, f)| names.contains(&f.name().as_str()))
            // contains is O(n) stinky, but schemas are never big anyway
            .map(|(idx, _)| idx)
    }

    /// Create a new [`Schema`] from the existing schema by selecting [`Field`]'s by name.
    pub fn select<'a>(&self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut selected = Vec::new();

        for name in names {
            let matches: Vec<FieldRef> = self
                .fields
                .iter()
                .filter(|f| f.name() == name.as_ref())
                .map(|f| f.clone())
                .collect();

            if matches.len() == 1 {
                selected.push(matches[0].clone());
            } else {
                panic!("multiple/or no matches in schema.select(), {:?}", matches)
            }
        }

        Self {
            fields: selected.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::datatypes::{DataType, Field};

    #[test]
    fn new() {
        let fields = vec![
            Field::new("kebab", DataType::Int32, true),
            Field::new("pizza", DataType::Float64, false),
        ];

        let s = Schema::from_fields(fields.clone());
        assert_eq!(s.fields, fields.into());
    }

    #[test]
    fn project_ok() {
        let fields = vec![
            Field::new("kebab", DataType::Int32, true),
            Field::new("pizza", DataType::Float64, false),
            Field::new("xd", DataType::Float64, false),
        ];
        let s = Schema::from_fields(fields.clone()).project([0, 2]);
        let expected = Schema::from_fields(vec![
            Field::new("kebab", DataType::Int32, true),
            Field::new("xd", DataType::Float64, false),
        ]);
        assert_eq!(expected, s)
    }

    #[test]
    #[should_panic]
    fn project_invalid_index() {
        let fields = vec![
            Field::new("kebab", DataType::Int32, true),
            Field::new("pizza", DataType::Float64, false),
        ];
        let _ = Schema::from_fields(fields.clone()).project([3]);
    }

    #[test]
    fn select_ok() {
        let fields = vec![
            Field::new("kebab", DataType::Int32, true),
            Field::new("pizza", DataType::Float64, false),
            Field::new("xd", DataType::Float64, false),
            Field::new("jockeboy", DataType::Float64, false),
        ];
        let s = Schema::from_fields(fields.clone()).select(["pizza", "jockeboy"]);
        let expected = Schema::from_fields(vec![
            Field::new("pizza", DataType::Float64, false),
            Field::new("jockeboy", DataType::Float64, false),
        ]);
        assert_eq!(expected, s)
    }

    #[test]
    #[should_panic]
    fn select_err() {
        let fields = vec![
            Field::new("kebab", DataType::Int32, true),
            Field::new("pizza", DataType::Float64, false),
            Field::new("xd", DataType::Float64, false),
            Field::new("jockeboy", DataType::Float64, false),
        ];
        let _ = Schema::from_fields(fields.clone()).select(["pizza", "elden_ring"]);
    }
}
