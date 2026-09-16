## Background

### Transparency logging

Transparency logging brings transparency to the way in which signing keys are used. No signature that
an end-user accepts as valid goes unnoticed, because it is included in a public log.

> Wait a second, I did not sign anything in the middle of the night. My key must be compromised.

The ability to say with confidence what signatures exist makes transparency logging a useful building block.
For example, consider an open-source software project that claims there are no secret releases. 
By incorporating the use of transparency logging, any release not listed on the project website can be detected.

> You claimed each release would be listed on the project website. Where is the release for signature 7d86…7730
> that appeared in the log? As you can see, it was created using your release key.

### Sigsum

Sigsum is the implementation of transparency logging that is utilized by the Mullvad app.

### The relay list

The relay list is a list of all the relays (VPN servers) that the app can connect to. 
The relay list is served from our backend API to enable us to dynamically update the list of relays that
the app can connect to after the app has been released. The app will periodically check for and
download new versions of the relay list in the background.

## Transparency logging the relay list

Every update to the relay list is logged in a public Sigsum transparency log. 
When the app fetches a new relay list from the server, it will only accept the new relay list if it
was logged in the Sigsum log with a signing key trusted by the app (the app is shipped
with the trusted keys bundled in).

### How transparency logging deters a would-be attacker

Even if an attacker were to compromise Mullvad's private key, the attacker has to publish a signature
of the compromised relay list to the Sigsum log for it to be accepted by the app.

Given that the attacker is not able to compromise the underlying Sigsum infrastructure,
this means that it is not possible to serve a modified or malicious relay list to a specific user
without publicly revealing that the tampered list was signed in the Sigsum log.

The fact that an attacker has to make the update that compromises the relay list public for all to see
serves as a deterrent against this type of attack.
