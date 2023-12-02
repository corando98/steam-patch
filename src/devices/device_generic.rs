use super::Device;
use crate::devices::Patch;
use crate::patch::{PatchFile, self};
use crate::server::SettingsRequest;
use crate::utils;
use crate::config::{get_global_config, self};

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
        let conf: config::Config = get_global_config();

        let mut patches = vec![
            Patch { //Sets max TDP
                text_to_find: "return[o,t,n,e=>a((()=>p.Get().SetTDPLimit(e)))".to_string(),
                replacement_text: format!("return[o,t,{:?},e=>a((()=>p.Get().SetTDPLimit(e)))", self.max_tdp).to_string(),
                destination: PatchFile::Chunk,
            },
            //Max GPU = 2700 //Self explanatory
            Patch { //Updated NOV16
                text_to_find: "return[o,t,n,e=>a((()=>p.Get().SetGPUPerformanceManualMhz(e)))".to_string(),
                replacement_text: format!("return[o,t,{:?},e=>a((()=>p.Get().SetGPUPerformanceManualMhz(e)))", self.max_gpu).to_string(),
                destination: PatchFile::Chunk,
            },
            // Listen to per app changes, required for QAM menu changes. 
            Patch {
                text_to_find: "const t=c.Hm.deserializeBinary(e).toObject();Object.keys(t)".to_string(),
                replacement_text: "const t=c.Hm.deserializeBinary(e).toObject(); console.log(t); fetch(`http://localhost:1338/update_settings`, { method: 'POST',  headers: {'Content-Type': 'application/json'}, body: JSON.stringify(t.settings)}); Object.keys(t)".to_string(),
                destination: PatchFile::Chunk,
            }, 
            //Overrides resolution for installed games so they are native resolution, must be installed with steam-patch patched in order for this change to go into effect.s
            Patch { //Nov 16
                text_to_find: "DownloadComplete_Title\"),r=Ue(n,t.data.appid());const s=(0,x.Q2)();".to_string(),
                replacement_text: "DownloadComplete_Title\"),r=Ue(n,t.data.appid()); SteamClient.Apps.GetResolutionOverrideForApp(t.data.appid()).then(res => res === \"Default\" && SteamClient.Apps.SetAppResolutionOverride(t.data.appid(), \"Native\")); const s=(0,x.Q2)();".to_string(),
                destination: PatchFile::Chunk, 
            },

        ];
        //Spoofing glyphs to match device
        if conf.spoof_glyphs {
            patches.extend(vec![
            // Replace Xbox menu button with Steam menu button
            Patch { //NOV16
                text_to_find: r#"e="/steaminputglyphs/xbox_button_logo.svg""#.to_string(),
                replacement_text: r#"return l.createElement(A.ActionGlyph, { button: n, size: A.EActionGlyphSize.Medium})"#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace PS menu button and ABXY 
            Patch { //NOV16
                text_to_find: r#"e="/steaminputglyphs/ps4_button_logo.svg""#.to_string(),
                replacement_text: r#"return l.createElement(A.ActionGlyph, { button: n, size: A.EActionGlyphSize.Medium})"#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button X -> A
            Patch { //NOV16
                text_to_find: r#""/steaminputglyphs/ps_color_button_x.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_a.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button Squeare -> X
            Patch { //NOV16
                text_to_find: r#""/steaminputglyphs/ps_color_button_square.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_x.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button Triagnle -> Y
            Patch { //NOV16
                text_to_find: r#""/steaminputglyphs/ps_color_button_triangle.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_y.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button Circle -> B
            Patch { 
                text_to_find: r#""/steaminputglyphs/ps_color_button_circle.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_b.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace option menu to sd
            Patch { 
                text_to_find: r#""/steaminputglyphs/ps4_button_options.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/sd_button_menu.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },

            // Replace button X -> A
            Patch { //NOV16
                text_to_find: r#""/steaminputglyphs/ps_button_x.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_a.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button Squeare -> X
            Patch { //NOV16
                text_to_find: r#""/steaminputglyphs/ps_button_square.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_x.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button Triagnle -> Y
            Patch { //NOV16
                text_to_find: r#""/steaminputglyphs/ps_button_triangle.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_y.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace button Circle -> B
            Patch { 
                text_to_find: r#""/steaminputglyphs/ps_button_circle.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/shared_button_b.svg""#.to_string(),
                destination: PatchFile::Chunk,
            },
            // Replace option menu to sd
            Patch { 
                text_to_find: r#""/steaminputglyphs/ps5_button_options.svg""#.to_string(),
                replacement_text: r#""/steaminputglyphs/sd_button_menu.svg""#.to_string(),
                destination: PatchFile::Chunk,
            }
            ]);
        }


        if conf.nintendo_glyphs {
            patches.extend(vec![
                Patch { //NOV16
                    text_to_find: r#"e="/steaminputglyphs/xbox_button_logo.svg""#.to_string(),
                    replacement_text: r#"return l.createElement(A.ActionGlyph, { button: n, size: A.EActionGlyphSize.Medium})"#.to_string(),
                    destination: PatchFile::Chunk,
                },
                // Replace PS menu button and ABXY 
                Patch { //NOV16
                    text_to_find: r#"e="/steaminputglyphs/ps4_button_logo.svg""#.to_string(),
                    replacement_text: r#"return l.createElement(A.ActionGlyph, { button: n, size: A.EActionGlyphSize.Medium})"#.to_string(),
                    destination: PatchFile::Chunk,
                },
                 // Replace button A -> B
                Patch { //NOV16
                    text_to_find: r#""/steaminputglyphs/shared_color_button_a.svg""#.to_string(),
                    replacement_text: r#""/steaminputglyphs/shared_button_b.svg""#.to_string(),
                    destination: PatchFile::Chunk,
                },
                // Replace button Squeare -> X
                Patch { //NOV16
                    text_to_find: r#""/steaminputglyphs/shared_color_button_b.svg""#.to_string(),
                    replacement_text: r#""/steaminputglyphs/shared_color_button_a.svg""#.to_string(),
                    destination: PatchFile::Chunk,
                },
                // Replace button Triagnle -> Y
                Patch { //NOV16
                    text_to_find: r#""/steaminputglyphs/shared_color_button_y.svg""#.to_string(),
                    replacement_text: r#""/steaminputglyphs/shared_button_x.svg""#.to_string(),
                    destination: PatchFile::Chunk,
                },
                // Replace button Squeare -> X
                Patch { //NOV16
                    text_to_find: r#""/steaminputglyphs/shared_color_button_x.svg""#.to_string(),
                    replacement_text: r#""/steaminputglyphs/shared_color_button_y.svg""#.to_string(),
                    destination: PatchFile::Chunk,
                },
                // Replace option menu to sd
                Patch { 
                    text_to_find: r#""/steaminputglyphs/ps4_button_options.svg""#.to_string(),
                    replacement_text: r#""/steaminputglyphs/sd_button_menu.svg""#.to_string(),
                    destination: PatchFile::Chunk,
                }
            ]);
        }
        //Return patches
        patches
    }

    fn get_key_mapper(&self) -> Option<tokio::task::JoinHandle<()>> {
        None
    }
}
