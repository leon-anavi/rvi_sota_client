use chan::Sender;
use rustc_serialize::json;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use datatype::{Config, Error, Event, Method, PendingUpdateRequest,
               UpdateRequestId, UpdateReport, UpdateReportWithDevice,
               UpdateResultCode, UpdateState, Url};
use http_client::{HttpClient, HttpRequest};


pub struct OTA<'c, 'h> {
    config: &'c Config,
    client: &'h HttpClient,
}

impl<'c, 'h> OTA<'c, 'h> {
    pub fn new(config: &'c Config, client: &'h HttpClient) -> OTA<'c, 'h> {
        OTA { config: config, client: client }
    }

    pub fn update_endpoint(&self, path: &str) -> Url {
        let endpoint = if path.is_empty() {
            format!("/api/v1/vehicle_updates/{}", self.config.device.uuid)
        } else {
            format!("/api/v1/vehicle_updates/{}/{}", self.config.device.uuid, path)
        };
        self.config.ota.server.join(&endpoint).unwrap()
    }

    pub fn get_package_updates(&mut self) -> Result<Vec<PendingUpdateRequest>, Error> {
        debug!("getting package updates");
        let resp_rx = self.client.send_request(HttpRequest {
            method: Method::Get,
            url:    self.update_endpoint(""),
            body:   None,
        });
        let resp = resp_rx.recv().expect("no get_package_updates response received");
        let data = try!(resp);
        let text = try!(String::from_utf8(data));
        Ok(try!(json::decode::<Vec<PendingUpdateRequest>>(&text)))
    }

    pub fn download_package_update(&mut self, id: &UpdateRequestId) -> Result<PathBuf, Error> {
        debug!("downloading package update");
        let resp_rx = self.client.send_request(HttpRequest {
            method: Method::Get,
            url:    self.update_endpoint(&format!("{}/download", id)),
            body:   None,
        });

        let mut path = PathBuf::new();
        path.push(&self.config.ota.packages_dir);
        path.push(id);
        // TODO: Use Content-Disposition filename from request?
        // TODO: Do not invoke package_manager
        path.set_extension(self.config.ota.package_manager.extension());

        let resp     = resp_rx.recv().expect("no download_package_update response received");
        let data     = try!(resp);
        let mut file = try!(File::create(path.as_path()));
        let _        = io::copy(&mut &*data, &mut file);
        Ok(path)
    }

    pub fn install_package_update(&mut self, id: &UpdateRequestId, etx: &Sender<Event>)
                                  -> Result<UpdateReport, Error> {
        debug!("installing package update");
        match self.download_package_update(id) {
            Ok(path) => {
                let err_str  = format!("Path is not valid UTF-8: {:?}", path);
                let pkg_path = try!(path.to_str().ok_or(Error::ParseError(err_str)));
                info!("Downloaded to {:?}. Installing...", pkg_path);

                // TODO: Fire DownloadComplete event, handle async UpdateReport command
                // TODO: Do not invoke package_manager
                let _ = etx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Installing));
                match self.config.ota.package_manager.install_package(pkg_path) {
                    Ok((code, output)) => {
                        let _ = etx.send(Event::UpdateStateChanged(id.clone(), UpdateState::Installed));
                        Ok(UpdateReport::single(id.clone(), code, output))
                    }

                    Err((code, output)) => {
                        let err_str = format!("{:?}: {:?}", code, output);
                        let _ = etx.send(Event::UpdateErrored(id.clone(), err_str));
                        Ok(UpdateReport::single(id.clone(), code, output))
                    }
                }
            }

            Err(err) => {
                etx.send(Event::UpdateErrored(id.clone(), format!("{:?}", err)));
                let failed = format!("Download failed: {:?}", err);
                Ok(UpdateReport::single(id.clone(), UpdateResultCode::GENERAL_ERROR, failed))
            }
        }
    }

    pub fn update_installed_packages(&mut self) -> Result<(), Error> {
        debug!("updating installed packages");
        // TODO: Fire GetInstalledSoftware event, handle async InstalledSoftware command
        // TODO: Do not invoke package_manager
        let pkgs = try!(self.config.ota.package_manager.installed_packages());
        let body = try!(json::encode(&pkgs));
        debug!("installed packages: {}", body);

        let resp_rx = self.client.send_request(HttpRequest {
            method: Method::Put,
            url:    self.update_endpoint("installed"),
            body:   Some(body.into_bytes()),
        });
        let _ = resp_rx.recv().expect("no update_installed_packages response received")
                       .map_err(|err| error!("update_installed_packages failed: {}", err));
        Ok(())
    }

    pub fn send_install_report(&mut self, report: &UpdateReport) -> Result<(), Error> {
        debug!("sending installation report");
        let vin_report = UpdateReportWithDevice::new(&self.config.device.uuid, &report);
        let body       = try!(json::encode(&vin_report));
        let resp_rx    = self.client.send_request(HttpRequest {
            method: Method::Post,
            url:    self.update_endpoint(&format!("{}", report.update_id)),
            body:   Some(body.into_bytes()),
        });
        let resp = resp_rx.recv().expect("no send_install_report response received");
        let _    = try!(resp);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use chan;
    use rustc_serialize::json;

    use super::*;
    use datatype::{Config, Event, Package, PendingUpdateRequest, UpdateResultCode, UpdateState};
    use http_client::TestHttpClient;
    use package_manager::PackageManager;
    use package_manager::tpm::assert_rx;


    #[test]
    fn test_get_package_updates() {
        let pending_update = PendingUpdateRequest {
            requestId: "someid".to_string(),
            installPos: 0,
            packageId: Package {
                name: "fake-pkg".to_string(),
                version: "0.1.1".to_string()
            },
            createdAt: "2010-01-01".to_string()
        };

        let json    = format!("[{}]", json::encode(&pending_update).unwrap());
        let mut ota = OTA {
            config: &Config::default(),
            client: &mut TestHttpClient::from(vec![json.to_string()]),
        };

        let updates: Vec<PendingUpdateRequest> = ota.get_package_updates().unwrap();
        let ids: Vec<String> = updates.iter().map(|p| p.requestId.clone()).collect();
        assert_eq!(ids, vec!["someid".to_string()])
    }

    #[test]
    fn bad_client_download_package_update() {
        let mut ota = OTA {
            config: &Config::default(),
            client: &mut TestHttpClient::new(),
        };
        let expect  = "Http client error: http://127.0.0.1:8080/api/v1/vehicle_updates/123e4567-e89b-12d3-a456-426655440000/0/download";
        assert_eq!(expect, format!("{}", ota.download_package_update(&"0".to_string()).unwrap_err()));
    }

    #[test]
    fn test_install_package_update_0() {
        let mut ota = OTA {
            config: &Config::default(),
            client: &mut TestHttpClient::new(),
        };
        let (tx, rx) = chan::async();
        let report   = ota.install_package_update(&"0".to_string(), &tx);
        assert_eq!(report.unwrap().operation_results.pop().unwrap().result_code,
                   UpdateResultCode::GENERAL_ERROR);

        let expect = r#"ClientError("http://127.0.0.1:8080/api/v1/vehicle_updates/123e4567-e89b-12d3-a456-426655440000/0/download")"#;
        assert_rx(rx, &[
            Event::UpdateErrored("0".to_string(), String::from(expect))
        ]);
    }

    #[test]
    fn test_install_package_update_1() {
        let mut config             = Config::default();
        config.ota.packages_dir    = "/tmp/".to_string();
        config.ota.package_manager = PackageManager::new_file(false);

        let mut ota = OTA {
            config: &config,
            client: &mut TestHttpClient::from(vec!["".to_string()]),
        };
        let (tx, rx) = chan::async();
        let report   = ota.install_package_update(&"0".to_string(), &tx);
        assert_eq!(report.unwrap().operation_results.pop().unwrap().result_code,
                   UpdateResultCode::INSTALL_FAILED);

        assert_rx(rx, &[
            Event::UpdateStateChanged("0".to_string(), UpdateState::Installing),
            // XXX: Not very helpful message?
            Event::UpdateErrored("0".to_string(), r#"INSTALL_FAILED: "failed""#.to_string())
        ]);
    }

    #[test]
    fn test_install_package_update_2() {
        let mut config = Config::default();
        config.ota.packages_dir    = "/tmp/".to_string();
        config.ota.package_manager = PackageManager::new_file(true);

        let replies = vec![
            "[]".to_string(),
            "package data".to_string(),
        ];
        let mut ota = OTA {
            config: &config,
            client: &mut TestHttpClient::from(replies),
        };
        let (tx, rx) = chan::async();
        let report   = ota.install_package_update(&"0".to_string(), &tx);
        assert_eq!(report.unwrap().operation_results.pop().unwrap().result_code,
                   UpdateResultCode::OK);

        assert_rx(rx, &[
            Event::UpdateStateChanged("0".to_string(), UpdateState::Installing),
            Event::UpdateStateChanged("0".to_string(), UpdateState::Installed)
        ]);
    }
}
