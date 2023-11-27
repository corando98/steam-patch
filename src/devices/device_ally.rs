use super::Device;
use crate::config::{get_global_config, self};
use crate::devices::device_generic::DeviceGeneric;
use crate::devices::Patch;
use crate::patch::PatchFile;
use crate::server::SettingsRequest;
use crate::steam::SteamClient;
use crate::{utils, main};
use std::fs::File;
use std::{fs, env};
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};
use dbus::blocking::Connection;
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;

pub struct DeviceAlly {
    device: DeviceGeneric,
}

impl DeviceAlly {
    pub fn new(tdp: i8, gpu: i16) -> Self {
        DeviceAlly {device: DeviceGeneric::new(tdp, 800,gpu)}
}
}

// Define the Platform trait for DBus interaction
trait Platform {
    fn set_ppt_pl1_spl(&self, value: u8) -> Result<(), dbus::Error>;
    fn set_ppt_pl2_sppt(&self, value: u8) -> Result<(), dbus::Error>;
    fn set_ppt_fppt(&self, value: u8) -> Result<(), dbus::Error>;
}
impl Platform for dbus::blocking::Proxy<'_, &Connection> {
    fn set_ppt_pl1_spl(&self, value: u8) -> Result<(), dbus::Error> {
        self.method_call("org.asuslinux.Daemon", "ppt_pl1_spl", (value,))
            .and_then(|response: ()| Ok(response))
    }
    // Update this based on return values from dbus interface
    
    fn set_ppt_pl2_sppt(&self, value: u8) -> Result<(), dbus::Error> {
        self.method_call("org.asuslinux.Daemon", "SetPptPl2Sppt", (value,))
            .and_then(|response: ()| Ok(response))
    }

    fn set_ppt_fppt(&self, value: u8) -> Result<(), dbus::Error> {
        self.method_call("org.asuslinux.Daemon", "SetPptFppt", (value,))
            .and_then(|response: ()| Ok(response))
    }
}



impl Device for DeviceAlly {
    fn set_thermalpolicy(&self, thermal_policy: i32) {
        // let conn = match Connection::new_system() {
        //     Ok(c) => c,
        //     Err(e) => {
        //         eprintln!("Failed to connect to DBus: {}", e);
        //         return;
        //     }
        // };
        // let proxy = conn.with_proxy(
        //     "org.asuslinux.Daemon", 
        //     "/org/asuslinux/Platform", 
        //     Duration::from_millis(5000),
        // );
        // match proxy.method_call(
        //     "org.asuslinux.Daemon", 
        //     "set_throttle_thermal_policy", 
        //     (thermal_policy,)
        // ) {
        //     Ok(_) => println!("Thermal policy set successfully."),
        //     Err(e) => eprintln!("Failed to set thermal policy: {}", e),
        // }
    }

    fn update_settings(&self, request: SettingsRequest) {
        if let Some(per_app) = &request.per_app {
            println!("{:#?}",per_app);
            // TDP changes
            if let Some(true) = per_app.is_tdp_limit_enabled {
                if let Some(tdp) = per_app.tdp_limit {
                    self.set_tdp(tdp);
                }
            }  else {
                self.set_thermalpolicy(1);
            }

            if let Some(gpu) = per_app.gpu_performance_manual_mhz {
                self.set_gpu(gpu);
            }
        }
    }
    //Add more patches for device specific
    fn get_patches(&self) -> Vec<Patch> {
        let mut patches = self.device.get_patches();
        patches.push(Patch {
            text_to_find: String::from("this.m_rgControllers=new Map,\"undefined\"!=typeof SteamClient&&(this.m_hUnregisterControllerDigitalInput"),
            replacement_text: String::from("this.m_rgControllers=new Map; window.HandleSystemKeyEvents = this.HandleSystemKeyEvents; \"undefined\"!=typeof SteamClient&&(this.m_hUnregisterControllerDigitalInput"),
            destination: PatchFile::Library,
        });
        patches
    }

    fn set_tdp(&self, tdp: i8) {
        let conn = Connection::new_system().unwrap();
        let proxy = conn.with_proxy("org.asuslinux.Daemon", "/org/asuslinux/Platform", Duration::from_millis(5000));


        let conf = get_global_config();
        if conf.legacy_tdp {
            self.device.set_tdp(tdp);
        } else { 
            let target_tdp = tdp as u8;
            let boost_tdp = target_tdp + 2;

            if let Err(e) = proxy.set_ppt_pl1_spl(target_tdp) {
                eprintln!("Failed to set ppt_pl1_spl: {}", e);
            }
    
            if let Err(e) = proxy.set_ppt_pl2_sppt(boost_tdp) {
                eprintln!("Failed to set ppt_pl2_sppt: {}", e);
            }
    
            if let Err(e) = proxy.set_ppt_fppt(target_tdp) {
                eprintln!("Failed to set ppt_fppt: {}", e);
            }
            
        }
    }

    fn set_gpu(&self, gpu: i16) {
        //Placeholder for later implementations
        println!("New GPU clock: {}", gpu);
    }

    fn get_key_mapper(&self) -> Option<tokio::task::JoinHandle<()>> {
        tokio::spawn(async move {
            let mut steam = SteamClient::new();
            steam.connect().await;
            start_mapper(steam);
        });
        None
    }
}

pub fn pick_device() -> Option<evdev::Device> {
    let target_vendor_id = 0xb05u16;
    let target_product_id = 0x1abeu16;

    let devices = evdev::enumerate();
    for (_, device) in devices {
        let input_id = device.input_id();

        if input_id.vendor() == target_vendor_id && input_id.product() == target_product_id {
            if device.supported_keys().map_or(false, |keys| keys.contains(evdev::Key::KEY_PROG1)) {
                return Some(device);   
            }
        }
    }
    None
}

pub fn recover_nkey() -> io::Result<()> {    
    // Check if a specific USB device is not present
    println!("ROG Ally detected and USB device 0b05:1abe not present");
    
    let command1 = format!("echo '\\_SB.PCI0.SBRG.EC0.CSEE' \"0xB7\" > /proc/acpi/call");
    let command2 = format!("echo '\\_SB.PCI0.SBRG.EC0.CSEE' \"0xB8\" > /proc/acpi/call");
    match utils::run_command(&[&command1]) {
        Ok(_) => println!("Set 0xB7"),
        Err(e) => println!("Couldn't set 0xB7 {}", e),
    }
    thread::sleep(Duration::from_secs(1));
    match utils::run_command(&[&command2]) {
        Ok(_) => println!("Set 0xB8"),
        Err(e) => println!("Couldn't set 0xB8 {}", e),
    }
    Ok(())
}

pub fn start_mapper(mut steam:SteamClient) -> Option<tokio::task::JoinHandle<()>> {
    let device = pick_device();
    let conf = get_global_config();
    conf.mapper;
    if conf.mapper {
    match device {
        Some(device) => Some(tokio::spawn(async move {
            if let Ok(mut events) = device.into_event_stream() {
                loop {
                    match events.next_event().await {
                        Ok(event) => {
                            if let evdev::InputEventKind::Key(key) = event.kind() {
                                // QAM button pressed
                                if key == evdev::Key::KEY_PROG1 && event.value() == 0 {
                                    println!("Show QAM");
                                    steam
                                        .execute("GamepadNavTree.m_Controller.OnButtonActionInternal(true, 28, 2)")
                                        .await;
                                }

                                // Main menu button pressed
                                if key == evdev::Key::KEY_F16 && event.value() == 0 {
                                    println!("Show Menu");
                                    steam
                                        .execute("GamepadNavTree.m_Controller.OnButtonActionInternal(true, 27, 2); console.log(\"Show Menu\");")
                                        .await;
                                }
                                
                                // Back button(s) (unified) Revisit once separated
                                if key == evdev::Key::KEY_F15 && event.value() == 0 {
                                    
                                    steam
                                        .execute("GamepadNavTree.m_Controller.OnButtonActionInternal(true, 26, 2); console.log(\"Simulating Rear right lower SteamDeck button\");")
                                        .await;
                                }
                            }
                        },
                        Err(_) => {
                            print!("Error reading event stream, retrying in 1 second");
                            thread::sleep(Duration::from_secs(1));
                            tokio::spawn(async move {
                                start_mapper(steam)
                            });
                            break
                        }
                    };
                }
            }
        })),
        None => {
            println!("No Ally-specific found, retrying in 2 seconds");

            thread::sleep(Duration::from_secs(2));
            if conf.auto_nkey_recovery {
                println!("N_key lost, attempting to trigger recovery script");
                let _ = recover_nkey();                 
            }
            tokio::spawn(async move {
                start_mapper(steam)
            });
            None
        }
    }
} else {
    None
}
}
