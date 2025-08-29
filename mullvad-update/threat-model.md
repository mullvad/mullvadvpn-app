# Introduction

This threat model describes the code backing Mullvad VPN loader and in-app updates on the two
platforms it supports (Windows and macOS). The loader is a graphical application used by Mullvad
users to install and upgrade the Mullvad VPN app on their devices, and in-app updates allows users
to update the app from within the app. The library crate `mullvad-update` is responsible for
verifying the integrity of the software that it downloads and installs on the user's device to
ensure that the software has not been tampered with. It allows the app to be hosted on untrusted
third-party CDNs without compromising security.

These tools perform network requests towards Mullvad API endpoints and above mentioned third-party
CDNs, and requires both read & write access to the target device file system.

## Acquiring Mullvad VPN loader

The loader application is initially downloaded from Mullvad’s website or the Mullvad VPN app GitHub
repository. For the installation artifacts on our website and GitHub, we provides detached PGP
signatures for integrity verification.

# Who do we trust

Some Mullvad employees - Access to publish metadata information to be consumed by `mullvad-update`
is segmented and has been granted to select individuals which are trusted within the company to make
app releases.


# Who is the attacker

## Nation states and law enforcement

With the goal of de-anonymizing individuals in order to track them and disarm “dissidents”.

## Crooks

With the goal to …

* Install malware on target devices

* Make our users part of botnets

* Steal users' information (crypto wallets etc)

# Capabilities of the attacker

* Changing what is served from the third party CDN network or the Mullvad API server

  * Serving malicious software or version metadata
  * Serving legitimate, but old versions of the version metadata or app binaries with known
    vulnerabilities
  * Serving files large enough to fill up the targets disk/ram

* Modify the downloaded installer on the client machine, tricking the `mullvad-update`
  mechanism to run a malicious installer with admin privileges. The result is that
  the attacker can escalate their foothold on the client machine from regular
  user to administrator.

# Countermeasures

Here are countermeasures we have identified against the above attackers which have been implemented
in `mullvad-update` and the loader/in-app upgrade mechanisms:

* Attach a signature to the metadata, and verify it on the client before using it

* Attach an expiry date to the signed part of the metadata, and don't use any expired metadata

* Attach an always increasing counter to the signed part of the metadata, and don't
  use any metadata with a lower counter than the highest previously observed valid counter

* Attach checksums of installer artifacts in the metadata, and verify that all downloaded artifacts
  has this expected checksum

* Attach the size of installer artifacts in the metadata, and abort any download if more than the
  expected amount of data is returned.

* Abort downloading the metadata if it is larger than a hardcoded max size

* Only allow trusted people to publish metadata via secured Qubes machines

* When relevant, only read/use downloaded software artifacts from a location that the loader (or
  admin) controls, to prevent privilege escalation


# Out of scope

* Most attacks involving physical access to the user's computer are not protected against.

* Malicious code that runs on the user's computer should not be able to use this software
  to escalate to higher privileges. But other than that, this threat model does
  not consider such an attacker.
