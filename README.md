# ⚙️ Steam Patch
Steam Patch is a tool designed to enhance your Steam experience by applying patches to the Steam client.

## Installation

```
curl -L https://github.com/corando98/steam-patch/raw/main/install.sh | sh
```


### GPU Slider not working? 

Edit the steamos-priv-write
```
sudo vi /usr/bin/steamos-polkit-helpers/steamos-priv-write
```
For Z1E devices (ROG ALLY, LEGION GO) around line 167, comment as such:
![Screenshot Terminal2023-12-18 at 16 51 @2x](https://github.com/corando98/steam-patch/assets/9966890/095f8002-9b53-473a-b418-1de71eb68c59)



## Updating

```
curl -L https://github.com/corando98/steam-patch/raw/main/install.sh | sh
```

## 📋 Available Patches

Here is a list of currently available patches that can be applied:

1. **TDP Slider Fix for Quick Access Menu**: This patch addresses and resolves the issues with the TDP slider in the Quick Access Menu, ensuring a smoother user experience.
   
2. **GPU Slider Fix for Quick Access Menu**: This patch integrates the QAM slider to the correct range, note: (ROG ONLY) changes to the ```steamos-priv-write``` file are required. 

3. **Menu Icon Replacement** For a more integrated and consistent look, this patch replaces <picture> <source media="(prefers-color-scheme: light)" srcset="https://github-production-user-asset-6210df.s3.amazonaws.com/5504685/255038062-d99f3be6-ff5a-4570-9f21-a59204ccc804.png"> <img src="https://github-production-user-asset-6210df.s3.amazonaws.com/5504685/255038464-eb72c683-a1a5-4e5c-b81a-0131f8a76dd7.png" height="20" align="center"> </picture> icon to <picture> <source media="(prefers-color-scheme: light)" srcset="https://github.com/Maclay74/steam-patch/assets/5504685/9d15c179-bb92-4463-9a06-f8faecccf5fe"> <img src="https://github.com/Maclay74/steam-patch/assets/5504685/c76f7637-9f82-4786-b936-0ee3d99039e3" height="20" align="center"> </picture>
4. **Mapping Device-Specific Buttons for Asus Rog Ally**: This patch adjusts the mapping of the Asus Rog Ally's device-specific buttons for the Main Menu and Quick Access Menu to match the button mapping of the Steam Deck.

## 🎯 Supported Devices

Offically tested devices include: ROG Ally, Legion Go

Generic features should work in most devices. 

## 🛠️ Configuration file

For customization a configuration is in place, find the config.toml in the root of the repo. Example below.

Default location is ~/steam-patch/config.toml

```
#For changes to reflect on steamUI, restart steam-patch, and restart steam. (Current mitigation)
main_enabled = true
tdp_control = true
gpu_control = true
max_tdp = 30               #If using smokeless bios you can push this higher
max_gpu = 2700

#Feature toggles
legacy_tdp = false         #true = ryzenadj, false = ACPI ROG ALLY METHOD
mapper = true              #Enable disable the QAM and Steam button mapping
spoof_glyphs = true        #Enable to change PS/Xbox buttons to match SD including Steam Menu and Option menu
nintendo_glyphs: false     #Swaps A <-> B and X <-> Y


#Experimental ROG ALLY ONLY
auto_nkey_recovery = false #Attempts to suspend and resume the device if NKEY is lost
```
Before adjusting the TDP, please ensure your device can support the new value. 
There is a tangible risk of causing damage to your device otherwise.


legacy_tdp - False, utilizes ryzenadj method of changing TDP, check if your device is compatible. ie. Legion Go, ROG Ally, etc
mapper - Only ROG ally for now, maps the QAM/Steam button to the AC/CC buttons.
auto_nkey_recovery - Extrememly hacky way of recoverying the AC/CC button due to sleep/suspend issue on Ally, use with caution.

## Steam Client

Only compatible with Stable Steam client, use beta branch for beta Steam client (breaks often)

## Credits

This project wouldn't have been possible without the work of Maclay74 for his base integration under the hood, thank you!

## 📝 License

This project is licensed under the [MIT License](LICENSE). Feel free to use and modify the code according to the terms of the license.

I've added a new section for supported devices and removed the note about no support of patching after a Steam restart as you requested.
