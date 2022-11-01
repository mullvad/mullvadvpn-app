# Mullvad VPN app build containers

Substitute `${repo}` with the actual absolute path to this repository

## Building and publishing a container image

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

Build and publish the container image. Tag it with the github hash of the current commit
```
git checkout -b update-build-container

./build-and-publish.sh (linux|android)

git push # And create a PR
```


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

