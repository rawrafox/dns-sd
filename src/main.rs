use std::sync::Arc;

use futures::StreamExt;
use kube::{Api, Client, Resource, ResourceExt};
use kube::runtime::controller::{Controller, Action};
use mdns_sd::{DaemonEvent, ServiceDaemon};
use tokio::time::Duration;
use tokio::spawn;

use dns_sd::ServiceInstance;

#[derive(thiserror::Error, Debug)]
pub enum Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;

async fn reconcile(obj: Arc<ServiceInstance>, mdns: Arc<ServiceDaemon>) -> Result<Action> {
  let meta = obj.meta();
  let info = obj.to_service_info().expect("Service info to be created");

  println!("registering services for {:?} in {} under {}", obj.name_any(), meta.namespace.as_deref().unwrap_or("global namespace"), info.get_hostname());
  println!(" - {}", info.get_fullname());

  mdns.register(info).expect("Failed to register our service");

  return Ok(Action::requeue(Duration::from_secs(3600)));
}

fn error_policy(_object: Arc<ServiceInstance>, _err: &Error, _mdns: Arc<ServiceDaemon>) -> Action {
  return Action::requeue(Duration::from_secs(5));
}

async fn monitor(mdns: Arc<ServiceDaemon>) {
  let monitor = mdns.monitor().expect("Monitoring to work");

  for event in monitor {
    match event {
      DaemonEvent::Announce(service, _) => println!("announced {}", service),
      DaemonEvent::IpAdd(address) => println!("added address {}", address),
      DaemonEvent::IpDel(address) => println!("removed address {}", address),
      DaemonEvent::Error(error) => println!("error: {:?}", error),
      unknown => println!("unknown event: {:?}", unknown)
    }
  }
}

async fn async_main(mdns: Arc<ServiceDaemon>) -> Result<(), kube::Error> {
  let client = Client::try_default().await?;
  let pods = Api::<ServiceInstance>::all(client);

  spawn(monitor(mdns.clone()));

  Controller::new(pods.clone(), Default::default())
    .run(reconcile, error_policy, mdns)
    .for_each(|_| futures::future::ready(()))
    .await;

  return Ok(());
}

fn main() -> Result<(), kube::Error> {
  let mdns = Arc::new(ServiceDaemon::new().expect("Failed to create daemon"));
  
  let rt = tokio::runtime::Runtime::new().unwrap();

  return rt.block_on(async_main(mdns));
}
