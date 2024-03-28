#!/usr/bin/env bash
 # inspired by https://github.com/actions/runner-images/blob/fb3b6fd69957772c1596848e2daaec69eabca1bb/images/macos/provision/configuration/configure-machine.sh#L33-L61
source ~/.bashrc

sudo security delete-certificate -Z FF6797793A3CD798DC5B2ABEF56F73EDC9F83A64 /Library/Keychains/System.keychain

# build utility to add certs
curl -o add-certificate.swift https://raw.githubusercontent.com/actions/runner-images/fb3b6fd69957772c1596848e2daaec69eabca1bb/images/macos/provision/configuration/add-certificate.swift
swiftc add-certificate.swift

sudo mv ./add-certificate /usr/local/bin/add-certificate
curl -o AppleWWDRCAG3.cer https://www.apple.com/certificateauthority/AppleWWDRCAG3.cer
curl -o DeveloperIDG2CA.cer https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
sudo add-certificate AppleWWDRCAG3.cer
sudo add-certificate DeveloperIDG2CA.cer
rm add-certificate* *.ce
sudo rm /usr/local/bin/add-certificate
