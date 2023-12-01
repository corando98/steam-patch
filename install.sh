#!/bin/bash

echo -e "Installing Steam Patch...\n"
cd $HOME
sudo rm -rf ./steam-patch/
git clone https://github.com/corando98/steam-patch
cd steam-patch
CURRENT_WD=$(pwd)

# Enable CEF debugging
touch "$HOME/.steam/steam/.cef-enable-remote-debugging"

which dnf 2>/dev/null
FEDORA_BASE=$?

cat /etc/nobara-release
NOBARA=$?

if [ $FEDORA_BASE == 0 ]; then
	echo -e '\nFedora based installation starting.\n'
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	sudo dnf install -y cargo
	mkdir -p $HOME/rpmbuild/{SPECS,SOURCES}
	cp steam-patch.spec $HOME/rpmbuild/SPECS
	rpmbuild -bb $HOME/rpmbuild/SPECS/steam-patch.spec
 	sudo dnf list --installed | grep steam-patch
  	STEAM_PATCH_STATUS=$?
   	if [ $STEAM_PATCH_STATUS == 0 ]; then
    		sudo dnf remove -y steam-patch
	fi
	sudo dnf install -y $HOME/rpmbuild/RPMS/x86_64/steam-patch*.rpm
fi

which pacman 2>/dev/null
ARCH_BASE=$?

cat /etc/os-release | grep ChimeraOS
CHIMERA_BASE=$?

if [ $ARCH_BASE == 0 ]; then
	echo -e '\nArch based installation starting.\n'
	if [ $CHIMERA_BASE == 0 ]; then
        	sudo frzr-unlock
	fi
	sudo pacman -Sy --noconfirm cargo gcc pkgconf
	printf "Installing steam-patch...\n"
	cargo build -r
	chmod +x $CURRENT_WD/target/release/steam-patch
	sudo cp $CURRENT_WD/target/release/steam-patch /usr/bin/steam-patch
	sed -i "s@\$USER@$USER@g" steam-patch.service
	sudo cp steam-patch.service /etc/systemd/system/
	sudo cp restart-steam-patch-on-boot.service /etc/systemd/system/
	sudo cp /usr/bin/steamos-polkit-helpers/steamos-priv-write /usr/bin/steamos-polkit-helpers/steamos-priv-write-bkp
	sudo cp steamos-priv-write-updated /usr/bin/steamos-polkit-helpers/steamos-priv-write
	# Start and enable services
	sudo systemctl daemon-reload
	sudo systemctl stop handycon
	sudo systemctl disable handycon
	sudo systemctl enable steam-patch.service
	sudo systemctl start steam-patch.service
	sudo systemctl enable restart-steam-patch-on-boot.service
	sudo systemctl start restart-steam-patch-on-boot.service
fi
