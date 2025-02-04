use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sparse::common::sparse_vector::SparseVector;
use validator::Validate;

use super::named_vectors::NamedVectors;
use crate::common::operation_error::OperationError;
use crate::common::utils::transpose_map_into_named_vector;
use crate::vector_storage::query::context_query::ContextQuery;
use crate::vector_storage::query::discovery_query::DiscoveryQuery;
use crate::vector_storage::query::reco_query::RecoQuery;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, rename_all = "snake_case")]
pub enum Vector {
    Dense(VectorType),
    Sparse(SparseVector),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VectorRef<'a> {
    Dense(&'a [VectorElementType]),
    Sparse(&'a SparseVector),
}

impl Vector {
    pub fn to_vec_ref(&self) -> VectorRef {
        match self {
            Vector::Dense(v) => VectorRef::Dense(v.as_slice()),
            Vector::Sparse(v) => VectorRef::Sparse(v),
        }
    }
}

impl Validate for Vector {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Vector::Dense(_) => Ok(()),
            Vector::Sparse(v) => v.validate(),
        }
    }
}

impl<'a> VectorRef<'a> {
    pub fn to_vec(self) -> Vector {
        match self {
            VectorRef::Dense(v) => Vector::Dense(v.to_vec()),
            VectorRef::Sparse(v) => Vector::Sparse(v.clone()),
        }
    }
}

impl<'a> TryFrom<VectorRef<'a>> for &'a [VectorElementType] {
    type Error = OperationError;

    fn try_from(value: VectorRef<'a>) -> Result<Self, Self::Error> {
        match value {
            VectorRef::Dense(v) => Ok(v),
            VectorRef::Sparse(_) => Err(OperationError::WrongSparse),
        }
    }
}

impl<'a> TryFrom<VectorRef<'a>> for &'a SparseVector {
    type Error = OperationError;

    fn try_from(value: VectorRef<'a>) -> Result<Self, Self::Error> {
        match value {
            VectorRef::Dense(_) => Err(OperationError::WrongSparse),
            VectorRef::Sparse(v) => Ok(v),
        }
    }
}

impl From<NamedVectorStruct> for Vector {
    fn from(value: NamedVectorStruct) -> Self {
        match value {
            NamedVectorStruct::Default(v) => Vector::Dense(v),
            NamedVectorStruct::Named(v) => Vector::Dense(v.vector),
            NamedVectorStruct::Sparse(v) => Vector::Sparse(v.vector),
        }
    }
}

impl TryFrom<Vector> for VectorType {
    type Error = OperationError;

    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        match value {
            Vector::Dense(v) => Ok(v),
            Vector::Sparse(_) => Err(OperationError::WrongSparse),
        }
    }
}

impl TryFrom<Vector> for SparseVector {
    type Error = OperationError;

    fn try_from(value: Vector) -> Result<Self, Self::Error> {
        match value {
            Vector::Dense(_) => Err(OperationError::WrongSparse),
            Vector::Sparse(v) => Ok(v),
        }
    }
}

impl<'a> From<&'a [VectorElementType]> for VectorRef<'a> {
    fn from(val: &'a [VectorElementType]) -> Self {
        VectorRef::Dense(val)
    }
}

impl<'a> From<&'a VectorType> for VectorRef<'a> {
    fn from(val: &'a VectorType) -> Self {
        VectorRef::Dense(val.as_slice())
    }
}

impl<'a> From<&'a SparseVector> for VectorRef<'a> {
    fn from(val: &'a SparseVector) -> Self {
        VectorRef::Sparse(val)
    }
}

impl From<VectorType> for Vector {
    fn from(val: VectorType) -> Self {
        Vector::Dense(val)
    }
}

impl From<SparseVector> for Vector {
    fn from(val: SparseVector) -> Self {
        Vector::Sparse(val)
    }
}

impl<'a> From<&'a Vector> for VectorRef<'a> {
    fn from(val: &'a Vector) -> Self {
        match val {
            Vector::Dense(v) => VectorRef::Dense(v.as_slice()),
            Vector::Sparse(v) => VectorRef::Sparse(v),
        }
    }
}

/// Type of vector element.
pub type VectorElementType = f32;

pub const DEFAULT_VECTOR_NAME: &str = "";

/// Type for dense vector
pub type VectorType = Vec<VectorElementType>;

impl<'a> VectorRef<'a> {
    // Cannot use `ToOwned` trait because of `Borrow` implementation for `Vector`
    pub fn to_owned(self) -> Vector {
        match self {
            VectorRef::Dense(v) => Vector::Dense(v.to_vec()),
            VectorRef::Sparse(v) => Vector::Sparse(v.clone()),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            VectorRef::Dense(v) => v.len(),
            VectorRef::Sparse(v) => v.indices.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> TryInto<&'a [VectorElementType]> for &'a Vector {
    type Error = OperationError;

    fn try_into(self) -> Result<&'a [VectorElementType], Self::Error> {
        match self {
            Vector::Dense(v) => Ok(v),
            Vector::Sparse(_) => Err(OperationError::WrongSparse),
        }
    }
}

impl<'a> TryInto<&'a SparseVector> for &'a Vector {
    type Error = OperationError;

    fn try_into(self) -> Result<&'a SparseVector, Self::Error> {
        match self {
            Vector::Dense(_) => Err(OperationError::WrongSparse),
            Vector::Sparse(v) => Ok(v),
        }
    }
}

pub fn default_vector(vec: Vec<VectorElementType>) -> NamedVectors<'static> {
    NamedVectors::from([(DEFAULT_VECTOR_NAME.to_owned(), vec)])
}

pub fn only_default_vector(vec: &[VectorElementType]) -> NamedVectors {
    NamedVectors::from_ref(DEFAULT_VECTOR_NAME, vec.into())
}

/// Full vector data per point separator with single and multiple vector modes
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(untagged, rename_all = "snake_case")]
pub enum VectorStruct {
    Single(VectorType),
    Multi(HashMap<String, Vector>),
}

impl VectorStruct {
    /// Check if this vector struct is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            VectorStruct::Single(vector) => vector.is_empty(),
            VectorStruct::Multi(vectors) => vectors.values().all(|v| match v {
                Vector::Dense(vector) => vector.is_empty(),
                Vector::Sparse(vector) => vector.indices.is_empty(),
            }),
        }
    }
}

impl Validate for VectorStruct {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            VectorStruct::Single(_) => Ok(()),
            VectorStruct::Multi(v) => common::validation::validate_iter(v.values()),
        }
    }
}

impl From<VectorType> for VectorStruct {
    fn from(v: VectorType) -> Self {
        VectorStruct::Single(v)
    }
}

impl From<&[VectorElementType]> for VectorStruct {
    fn from(v: &[VectorElementType]) -> Self {
        VectorStruct::Single(v.to_vec())
    }
}

impl<'a> From<NamedVectors<'a>> for VectorStruct {
    fn from(v: NamedVectors) -> Self {
        if v.len() == 1 && v.contains_key(DEFAULT_VECTOR_NAME) {
            let vector: &[_] = v.get(DEFAULT_VECTOR_NAME).unwrap().try_into().unwrap();
            VectorStruct::Single(vector.to_owned())
        } else {
            VectorStruct::Multi(v.into_owned_map())
        }
    }
}

impl VectorStruct {
    pub fn get(&self, name: &str) -> Option<VectorRef> {
        match self {
            VectorStruct::Single(v) => (name == DEFAULT_VECTOR_NAME).then_some(v.into()),
            VectorStruct::Multi(v) => v.get(name).map(|v| v.into()),
        }
    }

    pub fn into_all_vectors(self) -> NamedVectors<'static> {
        match self {
            VectorStruct::Single(v) => default_vector(v),
            VectorStruct::Multi(v) => NamedVectors::from_map(v),
        }
    }
}

/// Vector data with name
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub struct NamedVector {
    /// Name of vector data
    pub name: String,
    /// Vector data
    pub vector: VectorType,
}

/// Sparse vector data with name
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone, Validate)]
#[serde(rename_all = "snake_case")]
pub struct NamedSparseVector {
    /// Name of vector data
    pub name: String,
    /// Vector data
    #[validate]
    pub vector: SparseVector,
}

/// Vector data separator for named and unnamed modes
/// Unnamed mode:
///
/// {
///   "vector": [1.0, 2.0, 3.0]
/// }
///
/// or named mode:
///
/// {
///   "vector": {
///     "vector": [1.0, 2.0, 3.0],
///     "name": "image-embeddings"
///   }
/// }
#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum NamedVectorStruct {
    Default(VectorType),
    Named(NamedVector),
    Sparse(NamedSparseVector),
}

impl From<VectorType> for NamedVectorStruct {
    fn from(v: VectorType) -> Self {
        NamedVectorStruct::Default(v)
    }
}

impl From<NamedVector> for NamedVectorStruct {
    fn from(v: NamedVector) -> Self {
        NamedVectorStruct::Named(v)
    }
}

impl From<NamedSparseVector> for NamedVectorStruct {
    fn from(v: NamedSparseVector) -> Self {
        NamedVectorStruct::Sparse(v)
    }
}

pub trait Named {
    fn get_name(&self) -> &str;
}

impl Named for NamedVectorStruct {
    fn get_name(&self) -> &str {
        match self {
            NamedVectorStruct::Default(_) => DEFAULT_VECTOR_NAME,
            NamedVectorStruct::Named(v) => &v.name,
            NamedVectorStruct::Sparse(v) => &v.name,
        }
    }
}

impl NamedVectorStruct {
    pub fn new_from_vector(vector: Vector, name: String) -> Self {
        match vector {
            Vector::Dense(vector) => NamedVectorStruct::Named(NamedVector { name, vector }),
            Vector::Sparse(vector) => NamedVectorStruct::Sparse(NamedSparseVector { name, vector }),
        }
    }

    pub fn get_vector(&self) -> VectorRef {
        match self {
            NamedVectorStruct::Default(v) => v.as_slice().into(),
            NamedVectorStruct::Named(v) => v.vector.as_slice().into(),
            NamedVectorStruct::Sparse(v) => (&v.vector).into(),
        }
    }

    pub fn to_vector(self) -> Vector {
        match self {
            NamedVectorStruct::Default(v) => v.into(),
            NamedVectorStruct::Named(v) => v.vector.into(),
            NamedVectorStruct::Sparse(v) => v.vector.into(),
        }
    }
}

impl Validate for NamedVectorStruct {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            NamedVectorStruct::Default(_) => Ok(()),
            NamedVectorStruct::Named(_) => Ok(()),
            NamedVectorStruct::Sparse(v) => v.validate(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum BatchVectorStruct {
    Single(Vec<VectorType>),
    Multi(HashMap<String, Vec<Vector>>),
}

impl From<Vec<VectorType>> for BatchVectorStruct {
    fn from(v: Vec<VectorType>) -> Self {
        BatchVectorStruct::Single(v)
    }
}

impl BatchVectorStruct {
    pub fn into_all_vectors(self, num_records: usize) -> Vec<NamedVectors<'static>> {
        match self {
            BatchVectorStruct::Single(vectors) => vectors.into_iter().map(default_vector).collect(),
            BatchVectorStruct::Multi(named_vectors) => {
                if named_vectors.is_empty() {
                    vec![NamedVectors::default(); num_records]
                } else {
                    transpose_map_into_named_vector(named_vectors)
                }
            }
        }
    }
}

impl Validate for BatchVectorStruct {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            BatchVectorStruct::Single(_) => Ok(()),
            BatchVectorStruct::Multi(v) => {
                common::validation::validate_iter(v.values().flat_map(|batch| batch.iter()))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct NamedQuery<TQuery> {
    pub query: TQuery,
    pub using: Option<String>,
}

impl<T> Named for NamedQuery<T> {
    fn get_name(&self) -> &str {
        self.using.as_deref().unwrap_or(DEFAULT_VECTOR_NAME)
    }
}

impl<T: Validate> Validate for NamedQuery<T> {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        self.query.validate()
    }
}

#[derive(Debug, Clone)]
pub enum QueryVector {
    Nearest(Vector),
    Recommend(RecoQuery<Vector>),
    Discovery(DiscoveryQuery<Vector>),
    Context(ContextQuery<Vector>),
}

impl From<VectorType> for QueryVector {
    fn from(vec: VectorType) -> Self {
        Self::Nearest(Vector::Dense(vec))
    }
}

impl<'a> From<&'a [VectorElementType]> for QueryVector {
    fn from(vec: &'a [VectorElementType]) -> Self {
        Self::Nearest(Vector::Dense(vec.to_vec()))
    }
}

impl<const N: usize> From<[VectorElementType; N]> for QueryVector {
    fn from(vec: [VectorElementType; N]) -> Self {
        let vec: VectorRef = vec.as_slice().into();
        Self::Nearest(vec.to_owned())
    }
}

impl<'a> From<VectorRef<'a>> for QueryVector {
    fn from(vec: VectorRef<'a>) -> Self {
        Self::Nearest(vec.to_vec())
    }
}

impl From<Vector> for QueryVector {
    fn from(vec: Vector) -> Self {
        Self::Nearest(vec)
    }
}

impl From<SparseVector> for QueryVector {
    fn from(vec: SparseVector) -> Self {
        Self::Nearest(Vector::Sparse(vec))
    }
}
