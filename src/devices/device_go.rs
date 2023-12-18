use super::Device;
use crate::config::{get_global_config, self};
use crate::devices::device_generic::DeviceGeneric;
use crate::devices::Patch;
use crate::patch::PatchFile;
use crate::server::SettingsRequest;
use crate::steam::SteamClient;
use crate::{utils, main};
use std::fs::File as FFile;
use std::path::Path;
use std::{fs, env, vec};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration as DDuration;
use std::io::{self, Write, Read};
use std::io::BufRead;
use std::collections::HashMap;
use tokio::fs::{File, read_dir};
use tokio::io::AsyncReadExt;
use tokio::time::{timeout, Duration};

pub struct DeviceGo {
    device: DeviceGeneric,
}

impl DeviceGo {
    pub fn new(tdp: i8, gpu: i16) -> Self {
        DeviceGo {device: DeviceGeneric::new(tdp, 800,gpu)}
}
}

impl Device for DeviceGo {
    fn set_thermalpolicy(&self, thermalpolicy: i32){
        // The actual implementation would go here
        println!("Feature not implemented outside of ROG ALLY (Thermal policy): {}", thermalpolicy);
    }

    fn update_settings(&self, request: SettingsRequest) {
        let conf: config::Config = get_global_config();
        if let Some(per_app) = &request.per_app {
            println!("{:#?}",per_app);
            // TDP changes
            if let Some(true) = per_app.is_tdp_limit_enabled {
                if let Some(tdp) = per_app.tdp_limit {
                    self.set_tdp(tdp);
                }
            } else {
                if(conf.legacy_tdp == false){
                    let command_balanced = &format!("echo '\\_SB.GZFD.WMAA 0 0x2C {}' | sudo tee /proc/acpi/call; sudo cat /proc/acpi/call", 2);
                    match utils::run_command(&vec!["bash", "-c", command_balanced]) {
                        Ok(_) => println!("Changed power mode to balanced"),
                        Err(e) => println!("Ensure acpi_call module not loaded. {}", e),
                    }
                }
            }
            if let Some(gpu) = per_app.gpu_performance_manual_mhz {
                self.set_gpu(gpu);
            }
        }
    }
    //Add more patches for device specific
    fn get_patches(&self) -> Vec<Patch> {
        let mut patches = self.device.get_patches();
        print!("Adding patches for device specific");
        print!("{}", patches.len());
        patches.push(Patch {
            text_to_find: String::from("this.m_rgControllers=new Map,\"undefined\"!=typeof SteamClient&&(this.m_hUnregisterControllerDigitalInput"),
            replacement_text: String::from("this.m_rgControllers=new Map; window.HandleSystemKeyEvents = this.HandleSystemKeyEvents; \"undefined\"!=typeof SteamClient&&(this.m_hUnregisterControllerDigitalInput"),
            destination: PatchFile::Library,
        });
        patches
    }

    fn set_tdp(&self, tdp: i8) {
        let conf: config::Config = get_global_config();
        if conf.legacy_tdp {
            self.device.set_tdp(tdp);
        } else {
            print!("New TDP: {}", tdp);
            let command_custom = &format!("echo '\\_SB.GZFD.WMAA 0 0x2C {}' | sudo tee /proc/acpi/call; sudo cat /proc/acpi/call", 255);
            let target_tdp = tdp as i32;
            let boost_tdp = (target_tdp) + 2;
            let command_target = &format!("echo '\\_SB.GZFD.WMAE 0 0x12 {{0x00, 0xFF, 0x02, 0x01, {}, 0x00, 0x00, 0x00}}' | sudo tee /proc/acpi/call; sudo cat /proc/acpi/call", target_tdp);
            let command_boost = &format!("echo '\\_SB.GZFD.WMAE 0 0x12 {{0x00, 0xFF, 0x03, 0x01, {}, 0x00, 0x00, 0x00}}' | sudo tee /proc/acpi/call; sudo cat /proc/acpi/call", boost_tdp);
            let command_slow = &format!("echo '\\_SB.GZFD.WMAE 0 0x12 {{0x00, 0xFF, 0x01, 0x01, {}, 0x00, 0x00, 0x00}}' | sudo tee /proc/acpi/call; sudo cat /proc/acpi/call", target_tdp);
            let commands = vec![
                vec!["bash", "-c", command_custom], 
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

fn read_from_hidraw(device_path: &str, buffer_size: usize) -> io::Result<Vec<u8>> {
    let path = Path::new(device_path);
    let mut device = FFile::open(path)?;

    let mut buffer = vec![0u8; buffer_size];
    let bytes_read = device.read(&mut buffer)?;

    buffer.truncate(bytes_read);

    Ok(buffer)
}

pub async fn find_active_hidraw_device() -> io::Result<Option<String>> {
    let hidraw_base_path = "/sys/class/hidraw";
    let mut matching_devices = Vec::new();

    // Read the directory asynchronously
    let mut dir = read_dir(hidraw_base_path).await?;
    println!("Reading directory: {}", hidraw_base_path);

    // Iterate over the entries in the directory
    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        let uevent_path = path.join("device/uevent");
        // println!("Checking path: {:?}", uevent_path);

        if let Ok(mut uevent_file) = File::open(&uevent_path).await {
            let mut buffer = Vec::new();
            uevent_file.read_to_end(&mut buffer).await?;
            let contents = String::from_utf8_lossy(&buffer);
            // println!("Contents of file {}: {}", uevent_path.display(), contents);

            // Check if contents match your criteria 

            // Found X-input
            if contents.contains("Legion Controller for Windows") {
                if let Some(device_path) = path.file_name().map(|name| Path::new("/dev").join(name).to_string_lossy().into_owned()) {                
                    // println!("Matching device found: {:?}", device_path);
                    matching_devices.push(device_path);
                }
            }
            // Found D-input
            if contents.contains("Legion-Controller 1-A7") {
                if let Some(device_path) = path.file_name().map(|name| Path::new("/dev").join(name).to_string_lossy().into_owned()) {                
                    // println!("Matching device found: {:?}", device_path);
                    matching_devices.push(device_path);
                }
            }
        } else {
            println!("Could not open file: {:?}", uevent_path);
        }
    }
    println!("Trying the following devices: {:?}", matching_devices);
    let mut buffer = vec![0; 1024];
    
    for device_path in matching_devices.iter() {
        if let Ok(mut file) = File::open(device_path).await {
            // Set a timeout for the file.read operation
            let timeout_duration = Duration::from_secs(1); 
            let read_result = timeout(timeout_duration, file.read(&mut buffer)).await;
            println!("Now looking at device {:?}", device_path);
            println!("Read result: {:?}", read_result);
            
            match read_result {
                Ok(Ok(size)) if size == 64 => {
                    println!("Success at using {:?}", device_path);
                    return Ok(Some((device_path).to_string()));
                },
                _ => continue,
            }
        }
    }

    Ok(None)
}
pub fn start_mapper(mut steam: SteamClient) -> Option<tokio::task::JoinHandle<()>> {
    let conf = get_global_config();
    let buffer_size = 1024;

    if conf.mapper {
        Some(tokio::spawn(async move {
            let active_device = match find_active_hidraw_device().await {
                Ok(Some(path)) => path,
                _ => {
                    eprintln!("No active HIDRAW device found, retrying in 2 seconds");
                    tokio::time::sleep(Duration::from_secs(2)).await; // Asynchronous sleep
                    tokio::spawn(async move {
                        start_mapper(steam)
                    });
                    return;
                }
            };
            let mut previous_data = Vec::new(); // Variable to keep track of prev states
            loop {
                match read_from_hidraw(&active_device, buffer_size) {
                    Ok(data) => {
                        //Ensures that the data len is a whole packet of data
                        if previous_data != data && data.len() == 64{
                            // println!("Controller data: {:?}",data);
                            // println!("Data le {:?}", data.len());
                            if(data[18] == 64){
                                println!("Show QAM");
                                        steam
                                            .execute("GamepadNavTree.m_Controller.OnButtonActionInternal(true, 28, 2)")
                                            .await;
                            }
                            if(data[18] == 128){
                                if data[19] == 32 {
                                    println!("Show keyboard");
                                    steam.execute("SteamClient.URL.ExecuteSteamURL('steam://open/keyboard')").await;
                                } else {
                                    println!("Show Menu");
                                    steam.execute("GamepadNavTree.m_Controller.OnButtonActionInternal(true, 27, 2); console.log(\"Show Menu\");").await;
                                }
                            }
                            // if(data[18] == 128 && data[19] == 32) {
                            //     println!("Show keyboard");
                            //     steam.execute("SteamClient.URL.ExecuteSteamURL('steam://open/keyboard')").await;
                            // }
                        } else if data.len() < 64 {
                            println!("Device data length {:?}", data.len());
                            println!("Device path {:?}", &active_device);
                            // println!("Device data received: {:?}", data);
                        }
                        //Update prev state
                        previous_data = data.clone();
                    },
                    Err(e) => {
                        eprintln!("Failed to read from device: {}", e);                       
                        println!("Error reading event stream, retrying in 3 second");
                        thread::sleep(DDuration::from_secs(2));
                        tokio::spawn(async move {
                            start_mapper(steam)
                        });
                        break
                    },
                }
            }

        }))
    } else {
        println!("Mapper disabled");
        None
    }
}
