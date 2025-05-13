use crate::adapters::{dggrid::igeo7::Igeo7Impl, dggrid::isea3h::Isea3hImpl};
use crate::ports::dggrs::DggrsPort;
use std::sync::Arc;

pub fn get(tool: &str, dggrs: &str) -> Arc<dyn DggrsPort> {
    match (tool.to_uppercase().as_str(), dggrs.to_uppercase().as_str()) {
        ("DGGRID", "ISEA3H") => Arc::new(Isea3hImpl::default()),
        ("DGGRID", "IGEO7") => Arc::new(Igeo7Impl::default()),
        //("H3", "H3") => Arc::new(H3Impl),
        //("RHEALPIX", "RHEALPIX") => Arc::new(RhealpixImpl),
        _ => panic!(
            "Unsupported combination: tool='{}', dggrs='{}'",
            tool, dggrs
        ),
    }
}
