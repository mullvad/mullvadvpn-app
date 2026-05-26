# Background

## Transparency logging

Transparency logging brings transparency to the way in which signing keys are used. No signature that an end-user accepts as valid goes unnoticed because it is included in a __public log__.

> Wait a second, I did not sign anything in the middle of the night. My key must be compromised.

The ability to say with confidence what signatures exist makes transparency logging a useful building block. For example, consider an open-source software project that claims there are no secret releases. By incorporating the use of Sigsum, any release not listed on the project website can be detected.

> You claimed each release would be listed on the project website. Where is the release for signature 7d86…7730 that appeared in the log? As you can see it was created using your release key.

For security, Sigsum’s transparency has been designed to resist a powerful attacker that controls:

## Sigsum

[Sigsum](https://www.sigsum.org/) is an implementation of transparency logging has been designed to resist a powerful attacker that controls:

* The signer’s secret key and distribution infrastructure
* The public log, including its hosting infrastructure and secret key
* A threshold of so-called witnesses that call out if a log fails

## The relay list

The relay list is a list of all the relays (servers) that the app can connect to. The relay list if served from our backend
API to enable us to dynamically update the list of relays that the app can connect to after it has been released.
The app will periodically check for and download a new version of the relay list if one is available.

# Transparency logging the relay list

When a new version of the relay list is created, the backend will first take a hash of the relay list, sign the hash with
our private key and publish the hash to a Sigsum log. After this the witnesses that monitor the log will also sign the new log entry
because they trust our signing key.

## Downloading the relay list from the app

The app utilizes a two-step process to download the new relay list. First it makes a request to the `/trl/v0/timestamps/latest` 
endpoint. This returns the hash of the transparency logged relay list along with Sigsum metadata such as which public keys have
signed the Sigsum log entry. 

After this, the app validates that the hash it fetched was signed by Mullvad's signing key, as well as that it was signed by 
certain amount of witnesses that we trust. The exact policy for which and how many witnesses need to have signed the hash for it
to be accepted by the app is hardcoded into the app binary.

After the app has validated the relay list hash, it uses the hash to download the corresponding relay list 
from the `trl/v0/data` endpoint. 

# Who is the attacker

## Nation states and law enforcement

With the goal of de-anonymizing individuals in order to track them and disarm “dissidents”.

## Crooks

With the goal to …

* Make our users part of botnets

* Steal users' information (crypto wallets etc)

# Capabilities of the attacker

* Changing what is served from the Mullvad API server

* Access to Mullvad's signing private key

# Countermeasures

 * If the attacker is able to control the backend API server or act as a man-in-the-middle the attacker will not be
able to serve a compromised version of the relay list. This is because the app will reject any relay list update 
that has not been signed in accordance with the hardcoded Sigsum policy.
 
 * If the attacker, in addition to having control of the API, has access to Mullvad's private signing key the attacker
will still not be able to *directly* serve a compromised relay list. This is because the attacker does not have access to the 
witness' private signing keys. *However*, if the attacker has Mullvad's private key and are able to publish updates to the Sigsum
log they will be able to publish the hash of a compromised version of the relay list to the log. This entry will then be signed 
by the witnesses because they trust the Mullvad private key. Following this, the app will download and start using the compromised
relay list. In this scenario there is, however, a trace in the Sigsum log that a new relay list has been published to the log
when it shouldn't have been, which would hopefully be noticed by Mullvad or a third party.


# Out of scope

* Attackers that have the ability to install a compromised version of the Mullvad app on the user's device

* Attackers that have access to Mullvad's private key *and* the private keys of enough witnesses. If this is the case the attacker
can simply forge a Sigsum signature that the app will accept without the compromised relay list having ever been published 
to the Sigsum log.
