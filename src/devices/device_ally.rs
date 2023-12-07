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


pub struct DeviceAlly {
    device: DeviceGeneric,
}

impl DeviceAlly {
    pub fn new(tdp: i8, gpu: i16) -> Self {
        DeviceAlly {device: DeviceGeneric::new(tdp, 800,gpu)}
}
}

impl Device for DeviceAlly {
    fn set_thermalpolicy(&self, thermal_policy: i32) {
        println!("Setting new thermal policy: {}", thermal_policy);
        
        let file_path = "/sys/devices/platform/asus-nb-wmi/throttle_thermal_policy";
    
        // Attempt to write the thermal policy to the file.
        match fs::write(file_path, thermal_policy.to_string()) {
            Ok(_) => {
                // Optionally, add a small delay to give the system time to apply the setting.
                thread::sleep(Duration::from_millis(50));
    
                // Read the file back to confirm.
                match fs::read_to_string(file_path) {
                    Ok(content) if content.trim() == thermal_policy.to_string() => {
                        println!("Thermal policy set successfully.");
                    },
                    _ => {
                        eprintln!("Failed to set thermal policy. Value could not be confirmed.");
                    }
                }
            },
            Err(e) => {
                // Handle the error, but don't propagate it.
                eprintln!("Failed to write thermal policy: {}", e);
            }
        };
    }

    fn update_settings(&self, request: SettingsRequest) {
        if let Some(per_app) = &request.per_app {
            println!("{:#?}",per_app);
            // TDP changes
            if let Some(true) = per_app.is_tdp_limit_enabled {
                if let Some(tdp) = per_app.tdp_limit {
                    self.set_tdp(tdp);
                }
            }  
            else {
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
        // Update thermal policy
        let thermal_policy = match tdp {
            val if val < 9 => 2,                 // silent
            val if (9..=18).contains(&val) => 0, // performance
            _ => 1,                               // turbo
        };
        // < 9: Quiet thermal - 2
        // 9 - 18 : Balanced - 0
        // 19 - 30: Performance - 1

        self.set_thermalpolicy(thermal_policy); 

        let conf = get_global_config();
        if conf.legacy_tdp {
            self.device.set_tdp(tdp);
        } else { //Asus ROG ally 6.5+ kernel
            let target_tdp = tdp as i32;
            let boost_tdp = target_tdp + 2;
            let command_target = &format!("echo {} | sudo tee /sys/devices/platform/asus-nb-wmi/ppt_pl1_spl", target_tdp);
            let command_boost = &format!("echo {} | sudo tee /sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt", boost_tdp);
            let command_slow = &format!("echo {} | sudo tee /sys/devices/platform/asus-nb-wmi/ppt_fppt", target_tdp);
            println!("Using asus TDP method");
            let commands = vec![
                vec!["bash", "-c", command_target],
                vec!["bash", "-c", command_boost],
                vec!["bash", "-c", command_slow],
            ];
            for cmd in commands {
                println!("Command to run: {:?}",cmd);
                match utils::run_command(&cmd) {
                    Ok(_) => println!("Set TDP successfully!"),
                    Err(e) => println!("Couldn't set TDP: {}", e),
                }
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
