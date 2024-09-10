use kube::CustomResourceExt;

use dns_sd::ServiceInstance;

fn main() {
  print!("{}", serde_yaml::to_string(&ServiceInstance::crd()).unwrap());
}
