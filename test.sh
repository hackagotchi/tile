#!/bin/bash

rm *.AppImage
rm -rf Hexagotchi.AppDir
cargo build --release
mkdir -p Hexagotchi.AppDir
cp -r resources Hexagotchi.AppDir

cp target/release/tile Hexagotchi.AppDir/AppRun

cd Hexagotchi.AppDir
mv resources/logo.png hexagotchi.png

echo '[Desktop Entry]' > hexagotchi.desktop
echo 'Name=Hexagotchi' >> hexagotchi.desktop
echo 'Exec=Hexagotchi' >> hexagotchi.desktop
echo 'Icon=hexagotchi' >> hexagotchi.desktop
echo 'Type=Application' >> hexagotchi.desktop
echo 'Categories=Game;' >> hexagotchi.desktop

cd ..
./linuxdeploy-x86_64.AppImage Hexagotchi.AppDir
