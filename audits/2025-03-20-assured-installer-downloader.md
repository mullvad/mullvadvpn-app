# 2025-03-20 Assured's audit of the installer downloader

Between 2025-03-10 and 2025-03-17 [Assured Security Consultants] performed a review of
the distributed app download solution being developed by Mullvad VPN AB.

In scope for the audit was the installer downloader application, the script generating
installer releases, and the installer metadata.

Quoting the key conclusions and recommendations chapter of the report:

> Based on our review of the source code, the new downloader installer solution seems to be
> well thought out and implemented.
>
> Our observations has identified some minor issues and notes.
>
> Our recommendations can be summarized as follows:
>
> * Review the integrity check in the script `4-make-release`
> * Consider if the TOCTOU is really mitigated by the randomly generated directory name
> * Increase the number of characters in the randomly generated directory name

The [full audit report] can be found next to this file.

[Assured Security Consultants]: https://www.assured.se/
[full audit report]: ./2025-03-20-assured-MUL020_Installer_Downloader_Audit.pdf

## Overview of findings

This chapter will list the four findings from the report along with Mullvad's response
to them.

### 3.1 Release script does not verify that the key that has signed the binary is trusted (LOW)

The script we use to automate part of the desktop app release process (`4-make-release`) uses
`gpg --verify` to ensure that the installers it has fetched from Mullvad's internal release
servers are not tampered with.
Both the installer and the accompanying signature comes from the same server.

The audit report claims that `gpg --verify` will succeed as long as the signature matches the data,
no matter which key produced the signature. This claim is not fully accurate according
to our own tests. `gpg --verify` will exit with an exit code of zero if the signature
matches the data _and the key that produced the signature is in the local GnuPG keyring_.
Since we only import the Mullvad code signing key on the machines where we perform releases,
we think our usage of `gpg --verify` is currently pretty safe with the configuration
we are using. However, after discussing the matter in more detail with Assured we have together
identified two potential risks here:
1. GnuPG can be configured to automatically fetch keys from a keyserver when it verifies
   a signature it does not have the key for. This could render the check useless, but
   this behavior is currently disabled in the default configuration.
2. GnuPG can be configured to automatically use any public key bundled with the
   detached signature that it is verifying. This could render the signature check useless,
   but this behavior is also disabled by default.

The manpage for `gpg` explicitly states that only using the exit code, like we do, for signature
validation is not appropriate. So Assured is correct in pointing out this behavior as risky.
Both Assured and the manpage for `gpg` recommends `gpgv` as an alternative for usage in scripts.
However, even `gpgv` is a bit unclear about how it handles key servers and bundled pubkeys.
So instead we decided use [Sequoia] to verify the signatures, as it allows explicitly specifying
what PGP pubkey to trust. It also does not require us to initialize any keyring, so it is simpler
to use as well.

[Sequoia]: https://sequoia-pgp.gitlab.io/sequoia-sq/man/sq-verify.1.html


### 3.2 `deserialize_and_verify` function does not return the exact data that is signed and verified (NOTE)

This finding is about how we verify and use the signed metadata containing information about
available app versions. The design of this system is largely influenced by [The Update Framework].

The way the code worked at the time of the audit was roughly the following:

1. Download signed metadata JSON and deserialize to `partial_data`. This object contains
   the signature (`partial_data.signatures`) and the signed data (`partial_data.signed`).
2. Serialize `partial_data.signed` to Canonical JSON and call it `canon_data`.
3. Verify the signature against `canon_data`.
4. If the signature matched, return `partial_data.signed` and trust it.

The audit points out that if the library that performs serialization to canonical JSON has a bug
that translates `partial_data.signed` in the wrong way, we might pass verification, and then end
up using malicious data that was lost in the serialization process.
The recommendation here is to deserialize `canon_json` and return that as the trusted metadata.

A sidenote here is that the code was initially implemented according to the recommendation.
The code was changed to return `partial_data.signed` just before the audit. This change was
a result of a pre-audit meeting between Mullvad and Assured where we probably missunderstood
some of their early feedback on the metadata verification best practices.

We [changed the implementation back] to only use the verified data, as recommended.

[changed the implementation back]: https://github.com/mullvad/mullvadvpn-app/pull/7859/commits/1b6456794e1f784691f04a28540e4812eb6e7543
[The Update Framework]: https://theupdateframework.io/


### 3.3 Short random directory name (NOTE)

The macOS version of the installer downloader runs as the user who launched it. The program will
save the downloaded installer to a temporary directory writable by the user. The installer
downloader will then verify the checksum of the file and launch it if it matches. This leaves
a possible Time-of-Check, Time-of-Use (TOCTOU) attack vector. Any program running as the the same
user can replace the installer downloader between the time it was verified and it was launched.
Causing the installer downloader to launch a potentially malicious installer.

Mullvad was aware of this TOCTOU attack vector from the start. We did not classify it as within
the threat model. If malicious code runs as your user account, it could just as well have replaced
the installer downloader before it was even launched.

The audit report points out that we have written [code comments] saying that by storing the
installer in a directory with a randomized name, we mitigate the TOCTOU. This is an
unfortunate formulation in the documentation only. We (Mullvad) never meant that this fully
protects against the attack. It can make the attack slightly harder to carry out, but nothing more.

We have updated the documentation to not make invalid claims about its security properties:
[PR #7858] and [PR #7889]. We will not make further adjustments at this time, since the attack
is not in scope for the threat model.

[code comments]: https://github.com/mullvad/mullvadvpn-app/blob/1cb7935700827140f6430030033549c4d5cb2fb1/installer-downloader/src/temp.rs#L11-L17
[PR #7858]: https://github.com/mullvad/mullvadvpn-app/pull/7858
[PR #7889]: https://github.com/mullvad/mullvadvpn-app/pull/7889

### 3.4 `thread_rng()` is deprecated in latest rand version (NOTE)

The audit points out that we use `rand::thread_rng()` as random number generator, and that it is
deprecated in a version of the `rand` library newer than the one we are currently using.

There is nothing wrong or insecure about `thread_rng` in the version we are using, `rand`
just decided to rework the API a little bit for their `0.9.0` release.

We will make sure to use a secure CSPRNG once we upgrade this library in the future.


## Last words

We want to thank Assured for the valuable feedback on the update protocol and the professional
source code audit of the code related to the software update mechanisms.
