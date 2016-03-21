//! Sending side of the DBus interface.

use std::convert::From;

use dbus::{Connection, BusType, MessageItem, Message, FromMessageItem};

use configuration::DBusConfiguration;
use event::inbound::{UpdateAvailable, DownloadComplete, GetInstalledSoftware};
use event::outbound::{InstalledFirmwares, InstalledPackages, InstalledSoftware};

pub fn send_update_available(config: &DBusConfiguration, e: UpdateAvailable) {
    let args = [
        MessageItem::from(e.update_id),
        MessageItem::from(e.signature),
        MessageItem::from(e.description),
        MessageItem::from(e.request_confirmation)];
    let mut message = Message::new_method_call(
        &config.software_manager, &config.software_manager_path,
        &config.software_manager, "update_available").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let _ = conn.send(message)
        .map_err(|_| error!("Couldn't forward message to D-Bus"));
}

pub fn send_download_complete(config: &DBusConfiguration, e: DownloadComplete) {
    let args = [
        MessageItem::from(e.update_image),
        MessageItem::from(e.signature)];
    let mut message = Message::new_method_call(
        &config.software_manager, &config.software_manager_path,
        &config.software_manager, "download_complete").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let _ = conn.send(message)
        .map_err(|_| error!("Couldn't forward message to D-Bus"));
}

pub fn send_get_installed_software(config: &DBusConfiguration, e: GetInstalledSoftware)
    -> Result<InstalledSoftware, ()> {
    let args = [
        MessageItem::from(e.include_packages),
        MessageItem::from(e.include_module_firmware)];
    let mut message = Message::new_method_call(
        &config.software_manager, &config.software_manager_path,
        &config.software_manager, "get_installed_software").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let msg = conn.send_with_reply_and_block(message, config.timeout).unwrap();

    let mut args = msg.get_items().into_iter();
    let arg = try!(args.next().ok_or(()));
    let installed_packages: InstalledPackages = try!(FromMessageItem::from(&arg));

    let arg = try!(args.next().ok_or(()));
    let installed_firmware: InstalledFirmwares = try!(FromMessageItem::from(&arg));

    Ok(InstalledSoftware {
        packages: installed_packages,
        firmware: installed_firmware
    })
}

#[cfg(test)]
pub mod test {
    use dbus::{Connection, NameFlag, BusType, ConnectionItem, Message, MessageItem, FromMessageItem};
    use dbus::obj::*;
    use std::borrow::Cow;

    use configuration::DBusConfiguration;
    use event::UpdateId;
    use genivi::dbus::*;

    pub struct Swm {
        /// The configuration for the DBus interface.
        pub config: DBusConfiguration
    }

    impl Swm {
        pub fn start(&self) {
            let conn = Connection::get_private(BusType::Session).unwrap();
            conn.register_name(&self.config.software_manager, NameFlag::ReplaceExisting as u32).unwrap();

            let update_available = Method::new(
                "update_available",
                vec!(
                    Argument::new("update_id", "s"),
                    Argument::new("signature", "s"),
                    Argument::new("description", "s"),
                    Argument::new("request_confirmation", "s")),
                vec!(),
                Box::new(|msg| self.handle_update_available(msg)));
            let download_complete = Method::new(
                "download_complete",
                vec!(
                    Argument::new("update_image", "s"),
                    Argument::new("signature", "s")),
                vec!(),
                Box::new(|msg| self.handle_download_complete(msg)));
            let interface = Interface::new(vec!(update_available, download_complete), vec!(), vec!());

            let mut object_path = ObjectPath::new(&conn, &self.config.software_manager_path, true);
            object_path.insert_interface(&self.config.software_manager, interface);
            object_path.set_registered(true).unwrap();

            for n in conn.iter(1000) {
                match n {
                    ConnectionItem::MethodCall(mut m) => {
                        object_path.handle_message(&mut m);
                    },
                    _ => {}
                }
            }
        }

        fn handle_update_available(&self, msg: &mut Message) -> MethodResult {
            let mut args = msg.get_items().into_iter();
            let arg = try!(args.next().ok_or(missing_arg()));
            let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));

            self.send_initiate_download(update_id.clone());
            Ok(vec!())
        }

        fn handle_download_complete(&self, msg: &mut Message) -> MethodResult {
            let mut args = msg.get_items().into_iter();
            let arg = try!(args.next().ok_or(missing_arg()));
            let update_id: &String = try!(FromMessageItem::from(&arg).or(Err(malformed_arg())));

            self.send_update_report(update_id.clone());
            Ok(vec!())
        }

        fn send_initiate_download(&self, update_id: UpdateId) {
            let args = [MessageItem::from(update_id)];
            let mut message = Message::new_method_call(
                &self.config.name, &self.config.path,
                &self.config.interface, "initiate_download").unwrap();
            message.append_items(&args);

            let conn = Connection::get_private(BusType::Session).unwrap();
            conn.send(message)
                .map_err(|_| error!("Couldn't forward message to D-Bus"))
                .unwrap();
        }

        fn send_update_report(&self, update_id: UpdateId) {
            let args = [
                MessageItem::from(update_id),
                MessageItem::Array(vec!(), Cow::Borrowed("a{sv}"))];
            let mut message = Message::new_method_call(
                &self.config.name, &self.config.path,
                &self.config.interface, "update_report").unwrap();
            message.append_items(&args);

            let conn = Connection::get_private(BusType::Session).unwrap();
            conn.send(message)
                .map_err(|_| error!("Couldn't forward message to D-Bus"))
                .unwrap();
        }
    }
}
