//! Sending side of the DBus interface.

use std::convert::From;

use dbus::{Connection, BusType, MessageItem, Message, FromMessageItem};

use datatype::config::DBusConfiguration;
use datatype::update_request::{UpdateAvailable, DownloadComplete, GetInstalledSoftware};
use datatype::report::{InstalledFirmwares, InstalledPackages, InstalledSoftware};

pub fn send_update_available(config: &DBusConfiguration, e: UpdateAvailable) {
    let args = [
        MessageItem::from(e.update_id),
        MessageItem::from(e.signature),
        MessageItem::from(e.description),
        MessageItem::from(e.request_confirmation)];
    let mut message = Message::new_method_call(
        &config.software_manager, &config.software_manager_path,
        &config.software_manager, "updateAvailable").unwrap();
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
        &config.software_manager, "downloadComplete").unwrap();
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
        &config.software_manager, "getInstalledPackages").unwrap();
    message.append_items(&args);

    let conn = Connection::get_private(BusType::Session).unwrap();
    let msg = conn.send_with_reply_and_block(message, config.timeout).unwrap();

    let mut args = msg.get_items().into_iter();
    let arg = try!(args.next().ok_or(()));
    let installed_packages: InstalledPackages = try!(FromMessageItem::from(&arg));

    let arg = try!(args.next().ok_or(()));
    let installed_firmware: InstalledFirmwares = try!(FromMessageItem::from(&arg));

    Ok(InstalledSoftware::new(installed_packages, installed_firmware))
}

