use kube::CustomResource;
use mdns_sd::ServiceInfo;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(kind = "ServiceInstance", group = "dns-sd.aventine.se", version = "v1alpha1", namespaced)]
pub struct ServiceInstanceSpec {
  name: String,
  hostname: String,
  protocol: String,
  r#type: String,
  subtype: String,
  port: u16
}

impl ServiceInstance {
  pub fn to_service_info(&self) -> Result<ServiceInfo, mdns_sd::Error> {
    let spec = &self.spec;

    let info = ServiceInfo::new(
        &format!("_{}._{}.local.", spec.r#type, spec.protocol),
        &spec.name,
        &format!("{}.", spec.hostname),
        (),
        spec.port,
        None
    );

    return info.map(|e| e.enable_addr_auto());
  }
}