use serde::{Deserialize, Serialize};
use serde_json::Value;
use tardis::web::poem_openapi;

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalReq {
    pub kind: FlowExternalKind,
    pub curr_tag: String,
    pub curr_bus_obj_id: String,
    pub params: FlowExternalParams,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, poem_openapi::Enum)]
pub enum FlowExternalKind {
    FetchRelObj,
    ModifyField,
    NotifyChanges,
}

#[derive(Debug, Deserialize, Serialize, poem_openapi::Union)]
pub enum FlowExternalParams {
    FetchRelObj(FlowExternalFetchRelObjReq),
    ModifyField(FlowExternalModifyFieldReq),
    NotifyChanges(FlowExternalNotifyChangesReq),
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjReq {
    pub obj_tag: String,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalFetchRelObjResp {
    pub rel_bus_obj_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldReq {
    pub var_name: String,
    pub value: Option<Value>,
}

#[derive(Default, Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalModifyFieldResp {}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesReq {
    pub changed_vars: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, poem_openapi::Object)]
pub struct FlowExternalNotifyChangesResp {}
