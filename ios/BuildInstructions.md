# Create private key and Certificate Signing Request (CSR)

## Create new private key and CSR

OpenSSL will ask you the password for the private key, make sure to memorize it, you'll need it 
later.

```
openssl req -new -newkey rsa:2048 \
  -outform pem -keyform pem \
  -keyout private_key.pem \
  -out cert_signing_request \
  -subj "/C=SE/O=Amagicom AB/emailAddress=<YOUR_EMAIL>"
```

## [Alternative] Create CSR using the existing private key

```
openssl req -new \
  -outform pem -keyform pem \
  -key private_key.pem \
  -out cert_signing_request \
  -subj "/C=SE/O=Amagicom AB/emailAddress=<YOUR_EMAIL>"
```

# Upload Certificate Signing Request (CSR) to Apple

1. Go to https://developer.apple.com/account/resources/certificates/add
1. Select "Apple Distribution" option from the given list, press "Continue"
1. Select the previously created `cert_signing_request` file for upload
1. Download the provided `distribution.cer` on disk

# Download Apple WWDR certificate

WWDR certificate is used to verify the development and distribution certificates issued by Apple.

```
curl https://developer.apple.com/certificationauthority/AppleWWDRCA.cer > AppleWWDRCA.cer
```

# Convert certificates to PEM format

```
openssl x509 -inform der -outform pem \
  -in distribution.cer \
  -out distribution.pem

openssl x509 -inform der -outform pem \
  -in AppleWWDRCA.cer \
  -out AppleWWDRCA.pem
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
  -certfile AppleWWDRCA.pem \
  -out apple_code_signing.p12 \
  -name "Apple Distribution: Build Server (Amagicom AB)"
```

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

Note: providing the export passphrase using the `-P` flag is considered unsafe, however the default password prompt happens via GUI.

Technically after that you can clean up all created keys and certificates since all of them are 
securely stored in Keychain now.

```
rm distribution.{pem,cer} \
  AppleWWDRCA.{pem,cer} \
  cert_signing_request \
  apple_code_signing.p12 \
  private_key.pem
```

# Create iOS provisioning certificates

Create the provisioning profiles listed below, using Distribution > AppStore configuration.
When presented with the list of certificates, make sure to assign the provisioning profile to the 
same certificate (i.e `Amagicom AB (Apple Distribution)`) you received after uploading CSR.

| App ID                              | Provisioning Profile Name |
|-------------------------------------|---------------------------|
| net.mullvad.MullvadVPN              | Mullvad VPN Release       |
| net.mullvad.MullvadVPN.PacketTunnel | Packet Tunnel Release     |

Follow these steps to add each of provisioning profiles:

1. Go to https://developer.apple.com/account/resources/profiles/add
1. Choose "App Store" under "Distribution", then hit "Continue"
1. Choose the App ID (see the table above) and hit "Continue"
1. Choose the distribution certificate that you had created (i.e `Amagicom AB (Distribution)`)
1. Type in the profile name (see the table above) and hit "Generate"
1. Download the certificate in `ios/iOS Provisioning Profiles` directory. Create the directory if it 
   does not exist. 
   
   If you wish to customize the provisioning profiles location, provide the absolute path via 
   `IOS_PROVISIONING_PROFILES_DIR` environment variable.

# Setup AppStore credentials

For the automatic uploads to work properly, make sure to provide the Apple ID and password via
environment variables:

1. `IOS_APPLE_ID` - Your Apple ID (Email)
1. `IOS_APPLE_ID_PASSWORD` - Your Apple ID password or keychain reference (see explanation below)


`IOS_APPLE_ID_PASSWORD` accepts a keychain reference in form of `@keychain:<KEYCHAIN_ITEM_NAME>`.

Use app specific password instead of the actual account password and save it to Keychain. The app 
specific password can be created via AppStore Connect.

```
ALTOOL_BIN=$(xcrun -find altool)

security add-generic-password \
  -a <APPLE_ID_EMAIL> \
  -w <APP_SPECIFIC_PASSWORD> \
  -s <KEYCHAIN_ITEM_NAME> \
  -T "$ALTOOL_BIN"
```
# Automated build and deployment

Build script does not bump the build number, so make sure to do that manually and commit to repo:

```
agvtool bump
```

1. Run `./ios/build.sh` to build and export the app for upload to AppStore.
1. Run `./ios/build.sh --deploy` - same as above but also uploads the app to AppStore and 
                                   makes it available over TestFlight.

# Keychain quirks

It's possible that `codesign` will keep throwing the password prompts for Keychain, in that case try
the following commands __after__ importing the credentials into Keychain:

```
security unlock-keychain <KEYCHAIN>
security set-key-partition-list -S apple-tool:,apple: -s <KEYCHAIN>
```

where `<KEYCHAIN>` is the name of the target Keychain where the signing credentials are stored. 
This guide does not use a separate Keychain store, so use then use `login.keychain-db` then.

Reference: https://docs.travis-ci.com/user/common-build-problems/#mac-macos-sierra-1012-code-signing-errors
