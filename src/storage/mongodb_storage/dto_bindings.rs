use crate::dto;
use bson::serde_helpers::deserialize_hex_string_from_object_id;
use bson::serde_helpers::serialize_hex_string_as_object_id;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct Reqresp {
    #[serde(skip_serializing_if = "String::is_empty")]
    #[serde(serialize_with = "serialize_hex_string_as_object_id")]
    #[serde(deserialize_with = "deserialize_hex_string_from_object_id")]
    pub _id: String,
    pub req: dto::Request,
    pub resp: dto::Response,
}

impl From<dto::Reqresp> for Reqresp {
    fn from(value: dto::Reqresp) -> Self {
        return Reqresp {
            _id: value.id,
            req: value.req,
            resp: value.resp,
        };
    }
}

impl Into<dto::Reqresp> for Reqresp {
    fn into(self) -> dto::Reqresp {
        dto::Reqresp {
            id: self._id,
            req: self.req,
            resp: self.resp,
        }
    }
}
