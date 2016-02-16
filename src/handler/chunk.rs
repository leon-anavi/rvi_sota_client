//! Handles messages transferring single chunks.

use std::sync::Mutex;

#[cfg(not(test))] use rvi::send_message;

use message::{BackendServices, PackageId, ChunkReceived};
use handler::{Result, HandleMessageParams};
use persistence::Transfers;

/// Type for messages transferring single chunks.
#[derive(RustcDecodable)]
pub struct ChunkParams {
    /// The data of the transferred chunk.
    pub bytes: String,
    /// The index of this chunk.
    pub index: u64,
    /// The package transfer this chunk belongs to.
    pub package: PackageId
}

impl HandleMessageParams for ChunkParams {
    fn handle(&self,
              services: &Mutex<BackendServices>,
              transfers: &Mutex<Transfers>,
              rvi_url: &str,
              vin: &str) -> Result {
        let services = services.lock().unwrap();
        let mut transfers = transfers.lock().unwrap();
        transfers.get_mut(&self.package).map(|t| {
            if t.write_chunk(&self.bytes, self.index) {
                info!("Wrote chunk {} for package {}", self.index, self.package);
                send_message(rvi_url,
                             ChunkReceived {
                                 package: self.package.clone(),
                                 chunks: t.transferred_chunks.clone(),
                                 vin: vin.to_string()
                             },
                             &services.ack)
                    .map_err(|e| { error!("Error on sending ChunkReceived: {}", e); false })
                    .map(|_| None)
            } else {
                Err(false)
            }
        }).unwrap_or_else(|| {
            error!("Couldn't find transfer for package {}", self.package);
            Err(false)
        })
    }
}

#[cfg(test)]
use std::result;

#[cfg(test)]
fn send_message(url: &str, chunks: ChunkReceived, ack: &str)
    -> result::Result<bool, bool> {
    trace!("Would send received indices for {}, to {} on {}",
           chunks.package, ack, url);
    Ok(true)
}

#[cfg(test)]
mod test {
    use std::sync::Mutex;

    use super::*;
    use test_library::*;

    use rand;
    use rand::Rng;
    use rustc_serialize::base64;
    use rustc_serialize::base64::ToBase64;

    use handler::HandleMessageParams;
    use message::{BackendServices, PackageId};
    use persistence::{Transfer, Transfers};

    trait Tester<T> { fn new_test(i: usize, package: PackageId) -> T; }

    impl Tester<ChunkParams> for ChunkParams {
        fn new_test(i: usize, package: PackageId) -> ChunkParams {
            let msg = rand::thread_rng()
                .gen_ascii_chars().take(i).collect::<String>();
            let b64_msg = msg.as_bytes().to_base64(
                base64::Config {
                    char_set: base64::CharacterSet::UrlSafe,
                    newline: base64::Newline::LF,
                    pad: true,
                    line_length: None
                });

            ChunkParams {
                bytes: b64_msg,
                index: i as u64,
                package: package
            }
        }
    }

    #[test]
    fn it_returns_true_for_existing_transfers() {
        test_init!();
        for i in 1..20 {
            let prefix = PathPrefix::new();
            let mut transfer = Transfer::new_test(&prefix);
            let package = transfer.randomize(i);
            let transfers = Mutex::new(Transfers::new(prefix.to_string()));
            transfers.lock().unwrap().push_test(transfer);
            let services = Mutex::new(BackendServices::new());

            let chunk = ChunkParams::new_test(i, package);
            assert!(chunk.handle(&services, &transfers, "ignored", "").is_ok());
        }
    }

    #[test]
    fn it_returns_false_for_nonexisting_transfers() {
        test_init!();
        for i in 1..20 {
            let package = generate_random_package(i);
            let transfers = Mutex::new(Transfers::new("".to_string()));
            let services = Mutex::new(BackendServices::new());

            let chunk = ChunkParams::new_test(i, package);
            assert!(!chunk.handle(&services, &transfers, "ignored", "").is_ok());
        }
    }
}
