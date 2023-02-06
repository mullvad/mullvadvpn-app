# Create private key and Certificate Signing Request (CSR)

## Create new private key and CSR

OpenSSL will ask you the password for the private key, make sure to memorize it, you'll need it 
later.

```
openssl req -new -newkey rsa:2048 \
  -outform pem -keyform pem \
  -keyout private_key.pem \
  -out cert_signing_request \
  -subj "/C=SE/O=<ORGANIZATION_NAME>/emailAddress=<YOUR_EMAIL>"
```

## [Alternative] Create CSR using an existing private key

```
openssl req -new \
  -outform pem -keyform pem \
  -key private_key.pem \
  -out cert_signing_request \
  -subj "/C=SE/O=<ORGANIZATION_NAME>/emailAddress=<YOUR_EMAIL>"
```

# Upload Certificate Signing Request (CSR) to Apple

1. Go to https://developer.apple.com/account/resources/certificates/list
1. Click the plus button (+) in the heading to create a new certificate
1. Select "Apple Distribution" option from the given list, press "Continue"
1. Select the previously created `cert_signing_request` file for upload
1. Download the provided `distribution.cer` on disk

# Download Apple WWDR certificate

WWDR certificate is used to verify the development and distribution certificates issued by Apple.

```
curl https://www.apple.com/certificateauthority/AppleWWDRCAG3.cer -O
```

# Convert certificates to PEM format

```
openssl x509 -inform der -outform pem \
  -in distribution.cer \
  -out distribution.pem

openssl x509 -inform der -outform pem \
  -in AppleWWDRCAG3.cer \
  -out AppleWWDRCAG3.pem
```

# Export private key and certificates

Produce a PKCS12 container with the private key and all certificates. You will be asked to enter two
passphrases:

1. Private key passphrase
1. Export passphrase for PKCS12 file

You can store the produced `apple_code_signing.p12` file as a backup to be able to restore the keys
in the event of hardware failure. However you should always be able to re-create everything from 
scratch.

```
openssl pkcs12 -export \
  -inkey private_key.pem \
  -in distribution.pem \
  -certfile AppleWWDRCAG3.pem \
  -out apple_code_signing.p12 \
  -name "<FRIENDLY_KEYCHAIN_NAME>"
```

# Remove old private key and certificates from Keychain

__Skip this section if you create the private key for the very first time.__

If you happen to re-create the keys, you will have to remove the old keys and certificates from 
Keychain.

You can list all certificates with corresponding keys by using the following command:

```
security find-identity
```

You'll get a list of identities that looks like that:

```
Valid identities only
1) <HASH_ID> "Apple Distribution: <COMPANY NAME> (<TEAM ID>)"
2) <HASH_ID> "Apple Development: <COMPANY NAME> (<TEAM ID>)"
```

Pick the one that you don't want anymore and copy the `<HASH_ID>` from the output, then paste into 
the command below:

```
security delete-identity -Z <HASH_ID>
```

This should take care of removing both private keys and certificates. Repeat as many times as needed 
if you wish to remove multiple identities.

# Import private key and certificates into Keychain

```
security import apple_code_signing.p12 -f pkcs12 \
  -T /usr/bin/codesign \
  -P <EXPORT_PASSPHRASE>
```

Note: `-T /usr/bin/codesign` instructs Keychain to suppress password prompt during code signing, 
although you still have to unlock Keychain for that to have any effect. This instruction is 
equivalent to choosing "Always allow" in the password prompt GUI on the first run of `codesign` 
tool.

Note: providing the export passphrase using the `-P` flag is considered unsafe. 
Leave the `-P <EXPORT_PASSPHRASE>` out to enter the passphrase via GUI.

Technically after that you can clean up all created keys and certificates since all of them are 
securely stored in Keychain now.

```
rm distribution.{pem,cer} \
  AppleWWDRCAG3.{pem,cer} \
  cert_signing_request \
  apple_code_signing.p12 \
  private_key.pem
```

# Create iOS provisioning profiles

We will now create the provisioning profiles listed below using the Apple developer console.

| App ID                              | Provisioning Profile Name |
|-------------------------------------|---------------------------|
| net.mullvad.MullvadVPN              | Mullvad VPN Release       |
| net.mullvad.MullvadVPN.PacketTunnel | Packet Tunnel Release     |

Follow these steps to add each of provisioning profiles:

1. Go to https://developer.apple.com/account/resources/profiles/list
1. Click the plus button (+) in the heading to create a new provisioning profile
1. Choose "App Store" under "Distribution", then hit "Continue"
1. Choose the App ID (see the table above) and hit "Continue"
1. Choose the distribution certificate that you had created after uploading the CSR 
   (i.e `<ORGANIZATION_NAME> (Distribution)`)
1. Type in the profile name (see the table above) and hit "Generate"
1. Download the certificate in `ios/iOS Provisioning Profiles` directory. Create the directory if it 
   does not exist. 
   
   Note: you can use a different directory for storing provisioning profiles, however in that case,
   make sure to provide the path to the custom location via `IOS_PROVISIONING_PROFILES_DIR` 
   environment variable when running `build.sh` (more on that later).

# Setup AppStore credentials

For the automatic uploads to work properly, make sure to provide the Apple ID and password via
environment variables:

1. `IOS_APPLE_ID` - Your Apple ID (Email)
1. `IOS_APPLE_ID_PASSWORD` - Your Apple ID password or keychain reference (see explanation below)


`IOS_APPLE_ID_PASSWORD` accepts a keychain reference in form of `@keychain:<KEYCHAIN_ITEM_NAME>`.

Use the app specific password instead of the actual account password and save it to Keychain. 
The app specific password can be created via [Apple ID website] and added to Keychain using the 
following command (note that `altool` will be authorized to access the saved password):

```
xcrun altool --store-password-in-keychain-item <KEYCHAIN_ITEM_NAME> \
  -u <APPLE_ID_EMAIL> \
  -p "<APP_SPECIFIC_PASSWORD>"
```

[Apple ID website]: https://appleid.apple.com/account/manage

# Configure Xcode project

Copy template files of Xcode build configuration:

```
for file in ./ios/Configurations/*.template ; do cp $file ${file//.template/} ; done
```

Template files provide our team ID and correct provisioning profiles and generally do not require 
any changes when configuring our build server or developer machines for members of Mullvad 
development team. In all other cases perform the following steps to configure the project:

1. Edit `Base.xcconfig` and fill in your Apple development team ID, which can be found on Apple 
developer portal in the top right corner next to your organization name (uppercase letters and 
digits).
1. Edit `App.xcconfig` and `PacketTunnel.xcconfig` and supply the names of your provisioning profiles 
for development (Debug) and distribution (Release).
1. Edit `Screenshots.xcconfig` and supply the name of your provisioning profile. We only specify 
development profile here as we never build UI testing targets for distribution. Skip this step if 
you do not intend to generate screenshots for the app.

# Automated build and deployment

Build script does not bump the build number, so make sure to edit `Configurations/Version.xcconfig` 
and commit it back to repo.

1. Run `./ios/build.sh` to build and export the app for upload to AppStore.
1. Run `./ios/build.sh --deploy` - same as above but also uploads the app to AppStore and 
                                   makes it available over TestFlight.

# Keychain quirks

It's possible that `codesign` will keep throwing the password prompts for Keychain, in that case try
running the following commands __after__ importing the credentials into Keychain:

```
security unlock-keychain <KEYCHAIN>
security set-key-partition-list -S apple-tool:,apple: -s <KEYCHAIN>
```

where `<KEYCHAIN>` is the name of the target Keychain where the signing credentials are stored. 
This guide does not use a separate Keychain store, so use `login.keychain-db` then.

Reference: https://docs.travis-ci.com/user/common-build-problems/#mac-macos-sierra-1012-code-signing-errors

# SSL pinning

The iOS app utilizes SSL pinning. Root certificates can be updated by using the source certificates shipped along with `mullvad-api`:

```
openssl x509 -in ../mullvad-api/le_root_cert.pem -outform der -out MullvadREST/Assets/le_root_cert.cer
```
