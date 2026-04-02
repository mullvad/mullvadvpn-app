# How to contribute to the Mullvad VPN app

The Mullvad VPN app is open sourced for many reasons, but primarily we want to
* allow users to verify that our app functions as we claim it does, giving them the option to build
  it from source without having to trust our released binaries
* receive contributions from third parties.


## Filing issues

If you find a bug in the app's code, please report it on GitHub in the issue tracker. Please send
all other problems or questions (those not directly related to the app's development) to
[support@mullvadvpn.net](mailto:support@mullvadvpn.net). This includes connection issues, questions
regarding your account, and problems with the Mullvad VPN infrastructure or servers.


## Submitting changes

If you would like to contribute to the development of the Mullvad VPN app, please carefully read the
following sections first and then feel free to submit a pull request on GitHub.

While we appreciate your interest in helping us to improve Mullvad VPN, please understand that
choosing which submitted changes to merge is fully at our discretion, based upon our development
plans for the app. Before implementing new features, refactoring or making major changes, consider
existing [discussions](https://github.com/mullvad/mullvadvpn-app/discussions) or creating a
[new one](https://github.com/mullvad/mullvadvpn-app/discussions/new?category=feature-requests-ideas).
If you are fixing a bug, feel free to comment on the relevant existing issue or create a [bug report](https://github.com/mullvad/mullvadvpn-app/issues/new/choose)
before you start.

### AI-assisted contributions

A human must be behind all pull requests, issues, security advisories, and other submissions to this
repository. Regardless of whether AI was used to author the code, we require that a human
understands everything being submitted and is available for feedback and questions. If you have not
read your own changes or do not fully understand them, please do not expect us to either.

We will immediately close submissions where it is obvious that no human was in the loop, or where
the only communication we receive from the author is clearly from a bot.

### Localization / translations

The app is translated and proofread via a third party company. We can't take in user improvements
to the translations directly, since we can't verify their correctness. All translations have to
go via the translation company. As a result, if you want to improve an existing
translation, please don't edit the PO files and submit to us. Instead fill in your suggested
improvement in [this form], and the translation company will pick it up and process the
suggestion after a while.

[this form]: https://docs.google.com/forms/d/e/1FAIpQLSeEFRe0ojdl6QdHPp7Z9qIvdGTc1uSgbswQT6d-VRQ98GBO2w/viewform

### Copyright and ownership of contributed code and changes

Any code, binaries, tools, documentation, graphics, or any other material that you submit to this
project will be licensed under GPL 3.0. Submitting to this project means that you are the original
author of the entire contribution and grant us the full right to use, publish, change or remove
the entire, or part of, your contribution under the terms defined by the GPL 3.0 license at any
point in time.

### Code style and design

Please follow the [coding guidelines](https://github.com/mullvad/coding-guidelines).
