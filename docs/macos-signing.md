# How to generate keys and certificates for signing the macOS application and installer

## Generate private keys and CSRs

Generate *two* RSA keys and corresponding certificate signing requests (CSR).
One for "Developer ID Application" and one for "Developer ID Installer".
Replace C (country), CN (Common Name), O (Organization) and emailAddress as you see fit.

Yes, the keys must be 2048 bits. Apple won't allow 4096 for this.

```
openssl req -new -newkey rsa:2048 \
    -outform pem -keyform pem \
    -keyout private_key_application.pem \
    -out cert_signing_request_application \
    -subj "/C=SE/CN=Developer ID Application/O=Mullvad VPN AB/emailAddress=app@mullvad.net"

openssl req -new -newkey rsa:2048 \
    -outform pem -keyform pem \
    -keyout private_key_installer.pem \
    -out cert_signing_request_installer \
    -subj "/C=SE/CN=Developer ID Installer/O=Mullvad VPN AB/emailAddress=app@mullvad.net"
```

## Upload Certificate Signing Requests (CSR) to Apple

Do the following twice. Once for "Developer ID Application" and once for "Developer ID Installer",
and upload the respective CSR from the previous step.

1. Go to https://developer.apple.com/account/resources/certificates/list
1. Click the plus button (+) in the heading to create a new certificate
1. Select "Developer ID Application"/"Developer ID Installer" option from the given list, press "Continue"
1. Select `G2 Sub-CA (Xcode 11.4.1 or later)` under "Profile Type"
1. Select the previously created `cert_signing_request_application`/`cert_signing_request_installer` file for upload
1. Download the provided `developerID_application.cer`/`developerID_installer.cer`

## Download Apple IDG2CA intermediate certificate

The intermediate certificate that Apple used to sign the request above must be included in the pkcs12 file. So
download it. Apple might change which intermediate certificate they sign with. You can find it by looking
at the "Issuer" field from `openssl x509 -in developerID_application.pem -text`. (Follow the convert-to-pem
instructions below). All Apple's certificates are listed at https://www.apple.com/certificateauthority/.

```
curl -O https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
```

## Convert certificates from CER to PEM format

Convert all the DER (`.cer`) files obtained from Apple into PEM format that OpenSSL can handle better:
```
openssl x509 -inform der -outform pem -in developerID_application.cer -out developerID_application.pem
openssl x509 -inform der -outform pem -in developerID_installer.cer -out developerID_installer.pem
openssl x509 -inform der -outform pem -in DeveloperIDG2CA.cer -out DeveloperIDG2CA.pem
rm developerID_application.cer developerID_installer.cer DeveloperIDG2CA.cer
```

## Create PKCS12 containers with the private keys and certificates

Two PKCS12 containers are needed. One for application signing and one for installer signing.
Each PKCS12 (`.p12`) file will contain:
  * The private key
  * The certificate issued by Apple
  * The intermediate signing certificate downloaded from Apple (same in both `.p12` files)

When issuing this command you will be asked to provide the passphrase of the private key you
are trying to bundle, as well as the passphrase you want the PKCS12 container to have. These
should be the same password.

For the application signing key/cert:
```
openssl pkcs12 -export \
  -inkey private_key_application.pem \
  -in developerID_application.pem \
  -certfile DeveloperIDG2CA.pem \
  -out macos_signing_application.p12 \
```

Repeat the above command but replace "application" with "installer".

Only the `.p12` files will be used. So the remaining files can be removed if everything works
as intended.

## Using the signing keys/certificates

* Upload the `.p12` files to the macOS build server (unless it was created there).
* In the shell where the app is built, point to the `.p12` files so electron-builder can find them:
  ```
  export CSC_LINK=/path/to/macos_signing_application.p12
  export CSC_INSTALLER_LINK=/path/to/macos_signing_installer.p12
  ```
* The passprase for the `.p12` files must be assigned to `CSC_KEY_PASSWORD` and
  `CSC_INSTALLER_KEY_PASSWORD` respectively. But our `buildserver-build.sh` script will
  automatically ask for those, so no need to export them manually.


