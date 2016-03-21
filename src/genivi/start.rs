//! Main loop, starting the worker threads and wiring up communication channels between them.

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use configuration::Configuration;
use configuration::DBusConfiguration;
use event::Event;
use event::inbound::InboundEvent;
use event::outbound::OutBoundEvent;
use remote::svc::{RemoteServices, ServiceHandler};
use remote::rvi;

pub fn handle(cfg: &DBusConfiguration, rx: Receiver<Event>, remote_svcs: Arc<Mutex<RemoteServices>>) {
    loop {
        match rx.recv().unwrap() {
            Event::Inbound(i) => match i {
                InboundEvent::UpdateAvailable(e) => {
                    info!("UpdateAvailable");
                    super::swm::send_update_available(&cfg, e);
                },
                InboundEvent::DownloadComplete(e) => {
                    info!("DownloadComplete");
                    super::swm::send_download_complete(&cfg, e);
                },
                InboundEvent::GetInstalledSoftware(e) => {
                    info!("GetInstalledSoftware");
                    let _ = super::swm::send_get_installed_software(&cfg, e)
                        .and_then(|e| {
                            remote_svcs.lock().unwrap().send_installed_software(e)
                                .map_err(|e| error!("{}", e)) });
                }
            },
            Event::OutBound(o) => match o {
                OutBoundEvent::InitiateDownload(e) => {
                    info!("InitiateDownload");
                    let _ = remote_svcs.lock().unwrap().send_start_download(e);
                },
                OutBoundEvent::AbortDownload(_) => info!("AbortDownload"),
                OutBoundEvent::UpdateReport(e) => {
                    info!("UpdateReport");
                    let _ = remote_svcs.lock().unwrap().send_update_report(e);
                }
            }
        }
    }
}

/// Main loop, starting the worker threads and wiring up communication channels between them.
///
/// # Arguments
/// * `conf`: A pointer to a `Configuration` object see the [documentation of the configuration
///   crate](../configuration/index.html).
/// * `rvi_url`: The URL, where RVI can be found, with the protocol.
/// * `edge_url`: The `host:port` combination where the client should bind and listen for incoming
///   RVI calls.
pub fn start(conf: &Configuration, rvi_url: String, edge_url: String) {
    // Main message channel from RVI and DBUS
    let (tx, rx) = channel();

    // RVI edge handler
    let remote_svcs = Arc::new(Mutex::new(RemoteServices::new(rvi_url.clone())));
    let handler = ServiceHandler::new(tx.clone(), remote_svcs.clone(), conf.client.clone());
    let rvi_edge = rvi::ServiceEdge::new(rvi_url.clone(), edge_url, handler);
    rvi_edge.start();

    // DBUS handler
    let dbus_receiver = super::sc::Receiver::new(conf.dbus.clone(), tx);
    thread::spawn(move || dbus_receiver.start());
    handle(&conf.dbus, rx, remote_svcs);
}

#[cfg(test)]
mod test {
    use configuration::DBusConfiguration;
    use event::Event;
    use event::inbound::{UpdateAvailable, DownloadComplete, GetInstalledSoftware};
    use event::outbound::OutBoundEvent;
    use genivi;

    pub fn gen_cfg() -> DBusConfiguration {
        DBusConfiguration {
            name: "org.genivi.sota_client".to_string(),
            path: "/org/genivi/sota_client".to_string(),
            interface: "org.genivi.sota_client".to_string(),
            software_manager: "org.genivi.software_loading_manager".to_string(),
            software_manager_path: "/org/genivi/software_loading_manager".to_string(),
            timeout: 20
        }
    }

    #[test]
    fn it_sends_update_available() {
        use std::sync::mpsc::channel;
        use std::thread;

        let cfg = gen_cfg();
        let (tx, rx) = channel();
        let dbus_receiver = genivi::sc::Receiver::new(cfg.clone(), tx);
        thread::spawn(move || dbus_receiver.start());

        genivi::swm::send_update_available(
            &cfg,
            UpdateAvailable {
                update_id: "12345".to_string(),
                signature: "12345".to_string(),
                name: "test-update".to_string(),
                description: "test-update-desc".to_string(),
                request_confirmation: false,
                size: 1024
            });
        loop {
            match rx.recv() {
                Ok(e) => match e {
                    Event::OutBound(o) => match o {
                        OutBoundEvent::InitiateDownload(u) => genivi::swm::send_download_complete(
                            &cfg,
                            DownloadComplete {
                                update_id: u,
                                update_image: "/home/jerry/sota/client-device/sample_update.upd".to_string(),
                                signature: "1234".to_string()
                            }),
                        OutBoundEvent::AbortDownload(_) => info!("AbortDownload"),
                        OutBoundEvent::UpdateReport(_) => {
                            info!("UpdateReport");
                            break;
                        }
                    },
                    _ => {}
                },
                Err(e) => {
                    println!("Error: {}", e);
                    break;
                }
            }
        }
    }
}
