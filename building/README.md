# Mullvad VPN app build containers

Substitute `${repo}` with the actual absolute path to this repository

## Building and publishing a production container image

These instructions describe how to set up the trusted machine that builds, signs and publishes
the container images to ghcr.io.

If you `sudo` into a `build` account that do the builds, you need to set the permissions on the tty,
so podman can ask for the passphrase for the gpg key:
```
realuser@server $ sudo chown build:build $(tty)
realuser@server $ sudo -u build -i
```


Configure podman to store signatures when building and pushing images. `~/.config/containers/registries.d/mullvad.yaml`:
```yml
docker:
  ghcr.io/mullvad:
    sigstore-staging: file://${repo}/building/sigstore
```

Build and publish the container image. Tag it with the github hash of the current commit.
This also adds the container GPG signatures to the sigstore and commits that to git.
The single sigstore addition (signed) commit can be pushed directly to the main branch without PR.
```
./build-and-publish-container-image.sh (linux|android)
git push # Pushes the new sigstore entry
```

When satisfied with how the new image works, the `building/{linux,android}-container-image.txt`
files can be updated to point to the new image. The tag name of the new image is in the
commit message for the signed commit where the build server added the sigstore files.
This update is usually done in a separate PR by a developer

## Building and publishing a development image container image

These instructions describe how to set up a development machine to build, sign and publish container
images. The purpose of this is mainly to verify the `build-and-publish-container-image.sh`
script as well as the built images.

Set the following environment variables to override the default values:
- `REGISTRY_HOST`
- `REGISTRY_ORG`
- `CONTAINER_SIGNING_KEY_FINGERPRINT`

Configure podman to store signatures when building and pushing images (substitute `${testorg}`). `~/.config/containers/registries.d/$testorg.yaml`:

```yml
docker:
  ghcr.io/$testorg:
    sigstore-staging: file://${repo}/building/sigstore
```

In order to verify the signature of the development images, you'll also need to follow the [pull and
verification steps](#pulling-verifying-and-using-build-images) with some slight adjustments.

## Pulling, verifying and using build images

These instructions describe how anyone can pull the images and verify them with GPG before using them.

Copy the Mullvad app container signing GPG key to somewhere outside the repository (so a `git pull` can't overwrite it with a malicious key):
```
cp ${repo}/building/mullvad-app-container-signing.asc /path/to/mullvad-app-container-signing.asc
```

Configure a strict policy for podman when pulling from `ghcr.io/mullvad`. `~/.config/containers/policy.json`:
```json
{
  "default": [{ "type": "insecureAcceptAnything" }],
  "transports": {
    "docker": {
      "ghcr.io/mullvad": [
        {
          "type": "signedBy",
          "keyType": "GPGKeys",
          "keyPath": "/path/to/mullvad-app-container-signing.asc"
        }
      ]
    }
  }
}
```

Configure podman to fetch image signatures from the in-repo sigstore directory. `~/.config/containers/registries.d/mullvad.yaml`:
```yml
docker:
  ghcr.io/mullvad:
    sigstore: file://${repo}/building/sigstore
```

