# Repository security policy

Mullvad takes the security of our VPN app seriously. We perform third party security audits of the
entire app every second year. We also do smaller more specialized audits for certain features.
You can read more about these audits in the [audits directory](audits/README.md).

## Reporting security vulnerabilities

We welcome security researchers, customers or anyone else to scrutinize the source code of our
products and report any issues they find to us. We ask you to carry out responsible
research and disclosure. This includes, but is not limited to refraining from:

* Denial of service attacks against API endpoints used by the app
* Trying to disrupt the Mullvad VPN service
* Publicly disclosing vulnerabilities before reporting them to us in private.

Before reporting issues, we recommend that you read the following documents:
* [docs/security.md] - Explaining various expected security properties of the app
* [known issues] - Listing already known issues in the app.

**Please do not report security vulnerabilities through GitHub issues or other
public channels.** Instead please [create a vulnerability report on Github]. Or email our
support on [support@mullvadvpn.net]. Preferrably encrypted with our [support's PGP] key.

[create a vulnerability report on Github]: https://github.com/mullvad/mullvadvpn-app/security/advisories/new
[support@mullvadvpn.net]: mailto:support@mullvadvpn.net
[support's PGP]: https://mullvad.net/static/gpg/mullvadvpn-support-mail.asc
[known issues]: docs/known-issues.md
[docs/security.md]: docs/security.md
