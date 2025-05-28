# Introduction

This threat model describes the Mullvad VPN loader on the two platforms it supports (Windows and
macOS). The loader is used by Mullvad users to install and upgrade the Mullvad VPN app on their
devices. It is responsible for verifying the integrity of the software that it downloads and
installs on the user's device to ensure that the software has not been tampered with. It allows the
app to be hosted on untrusted third-party CDNs without compromising security.

The loader performs network requests towards Mullvad API endpoints and above mentioned third-party
CDNs, and requires both read & write access to the target device file system.

## Acquiring Mullvad VPN loader

The application is either downloaded from Mullvad’s website or the Mullvad VPN app GitHub
repository. For the installation artifacts on our website and GitHub, we provides detached PGP
signatures for integrity verification.

# Who do we trust

Some Mullvad employees - Access to publish metadata information to be consumed by the loader is
segmented and has been granted to select individuals which are trusted within the company to make
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

* Serving malicious software via the third-party CDNs

* Serving legitimate old or unexpected versions of the app on third-party CDNs, e.g.
  downgrading to versions with known vulnerabilities or development builds

* Serving files large enough to fill up the target's disk

* Compromising the Mullvad API, and (e.g.) returning outdated or fake version metadata

* Having physical access to the target device

# Countermeasures

Here are countermeasures we have identified against the above attackers which have been implemented
in the loader:

* The version metadata / Mullvad API response is cryptographically verified to be signed

* The version metadata has an expiry date

* The checksum of software packages downloaded via third-party CDNs is cryptographically verified to
  be the same as the checksum in the metadata

* Only allow trusted people to publish metadata via secured Qubes machines

* On Windows, only read/use downloaded software artifacts from a location that the loader (or
  admin) controls, to prevent privilege escalation

* The size of the downloaded software package is checked to be the correct size, and if larger the
  download is aborted

# Out of scope

* Most attacks involving physical access to the user's computer are not covered by the threat model

* Malicious code that runs as your user account

* Attacks against the app installer are not covered by this threat model
