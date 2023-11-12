use super::Device;
use crate::devices::Patch;
use crate::patch::PatchFile;
use crate::server::SettingsRequest;
use crate::utils;

pub struct DeviceGeneric {
    max_tdp: i8,
    max_gpu: i16,
    min_gpu: i16,
}

impl DeviceGeneric {
    pub fn new(max_tdp: i8, min_gpu: i16, max_gpu: i16) -> DeviceGeneric {
        DeviceGeneric { max_tdp, max_gpu, min_gpu}
    }
}

impl Device for DeviceGeneric {
    fn update_settings(&self, request: SettingsRequest) {
        if let Some(per_app) = &request.per_app {
            // TDP changes
            if let Some(tdp) = per_app.tdp_limit {
                self.set_tdp(tdp);
            }
            //GPU Clock changes
            if let Some(gpu) = per_app.gpu_performance_manual_mhz {
                self.set_gpu(gpu);
            }
        }
    }
    fn set_thermalpolicy(&self, thermalpolicy: i32){
        // The actual implementation would go here
        println!("Feature not implemented outside of ROG ALLY (Thermal policy): {}", thermalpolicy);
    }

    fn set_tdp(&self, tdp: i8) {
        // Update TDP
        let target_tdp = tdp as i32 * 1000;
        let boost_tdp = target_tdp + 2000;
        let mut command: Vec<String>;
        // echo 30 | sudo tee /sys/devices/platform/asus-nb-wmi/ppt_pl1_spl
        // echo 43 | sudo tee /sys/devices/platform/asus-nb-wmi/ppt_pl2_sppt
        // echo 53 | sudo tee /sys/devices/platform/asus-nb-wmi/ppt_fppt
        command = vec![
            "ryzenadj".to_string(),
            format!("--stapm-limit={}", target_tdp),
            format!("--fast-limit={}", boost_tdp),
            format!("--slow-limit={}", target_tdp),
        ]; 
        
        let command_strs: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
        println!("Command to run: {:?}",command);
        match utils::run_command(&command_strs) {
            Ok(_) => println!("Set TDP successfully!"),
            Err(_) => println!("Couldn't set TDP"),
        }
    }

    fn set_gpu(&self, gpu: i16) {
        println!("Setting GPU to {}", gpu);
    }

    fn get_patches(&self) -> Vec<Patch> {
        vec![
            // Max TDP = 28
            Patch {
                text_to_find: "return[o,t,n,e=>a((()=>p.Get().SetTDPLimit(e)))".to_string(),
                replacement_text: format!("return[o,t,{:?},e=>a((()=>p.Get().SetTDPLimit(e)))", self.max_tdp).to_string(),
                destination: PatchFile::Chunk,
            },
            //Max GPU = 2700
            Patch {
                text_to_find: "return[o,t,n,e=>a((()=>p.Get().SetGPUPerformanceManualMhz(e)))".to_string(),
                replacement_text: format!("return[o,t,{:?},e=>a((()=>p.Get().SetGPUPerformanceManualMhz(e)))", self.max_gpu).to_string(),
                destination: PatchFile::Chunk,
            },
            // Listen to per app changes
            Patch {
                text_to_find: "const t=c.Hm.deserializeBinary(e).toObject();Object.keys(t)".to_string(),
                replacement_text: "const t=c.Hm.deserializeBinary(e).toObject(); console.log(t); console.log(W); fetch(`http://localhost:1338/update_settings`, { method: 'POST',  headers: {'Content-Type': 'application/json'}, body: JSON.stringify(t.settings)}); Object.keys(t)".to_string(),
                destination: PatchFile::Chunk,
            }, 
            Patch {
                text_to_find: "s.k_EControllerTypeFlags_XBox360".to_string(),
                replacement_text: "s.k_EControllerTypeFlags_SteamControllerNeptune".to_string(),
                destination: PatchFile::Chunk,
            }, 
            
            // Replace Xbox menu button with Steam one CAUSING CRASH
            // Raw literal strings with escape for REGEX
            Patch {
                text_to_find: r#"e="/steaminputglyphs/xbox_button_logo.svg""#.to_string(),
                replacement_text: r#"return l.createElement(A.ActionGlyph, { button: n, size: A.EActionGlyphSize.Medium})"#.to_string(),
                
                // style=\"width: 61.1px; height: auto; margin: -100% 0 -100% 0;\"#.to_string(),
                destination: PatchFile::Chunk,
            },

            // Change resolution to Native (if Default) after installation DISABLED UNTIL PROPER METHOD
            // Patch {
            //     text_to_find: "DownloadComplete_Title\"),i=Ue(n,t.data.appid());const s=(0,x.Q2)();".to_string(),
            //     replacement_text: "DownloadComplete_Title\"),i=Ue(n,t.data.appid()); SteamClient.Apps.GetResolutionOverrideForApp(t.data.appid()).then(res => res === \"Default\" && SteamClient.Apps.SetAppResolutionOverride(t.data.appid(), \"Native\")); const l=(0,H.Q2)();".to_string(),
            //     destination: PatchFile::Chunk, 
            // },
        ]
    }

    fn get_key_mapper(&self) -> Option<tokio::task::JoinHandle<()>> {
        None
    }
}
