use astral_yaml::schema::{self, ScalarType};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum YamlType {
    Scalar(ScalarType),
    Sequence,
    Mapping,
}

impl YamlType {
    pub(crate) fn matches_scalar(self, value: &str) -> bool {
        match self {
            Self::Scalar(ScalarType::String) => true,
            Self::Scalar(ScalarType::Float) => {
                matches!(schema::classify(value), ScalarType::Float | ScalarType::Int)
                    && !(value.starts_with("0x") || value.starts_with("0o"))
            }
            Self::Scalar(kind) => schema::classify(value) == kind,
            Self::Sequence | Self::Mapping => false,
        }
    }

    pub(crate) fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "tag:yaml.org,2002:str" => Some(Self::Scalar(ScalarType::String)),
            "tag:yaml.org,2002:null" => Some(Self::Scalar(ScalarType::Null)),
            "tag:yaml.org,2002:bool" => Some(Self::Scalar(ScalarType::Bool)),
            "tag:yaml.org,2002:int" => Some(Self::Scalar(ScalarType::Int)),
            "tag:yaml.org,2002:float" => Some(Self::Scalar(ScalarType::Float)),
            "tag:yaml.org,2002:seq" => Some(Self::Sequence),
            "tag:yaml.org,2002:map" => Some(Self::Mapping),
            _ => None,
        }
    }

    pub(crate) fn accepts(self, declared: Self) -> bool {
        // Floating-point destinations also accept YAML integers.
        self == declared
            || matches!(
                (self, declared),
                (
                    Self::Scalar(ScalarType::Float),
                    Self::Scalar(ScalarType::Int)
                )
            )
    }
}
