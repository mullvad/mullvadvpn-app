# 2026-02-17 - Leviathan security assessment of our Android app

[Leviathan Security Group] conducted a Mobile Application Security Assessment (MASA) of our Android
app. It's a standardized assessment that checks an app against a specification called Mobile
Application Profile (MAP).

## Overview of findings

### 1.5.1.4 All Pending Intents shall be immutable or otherwise justified for mutability

We agree with the assessment of Leviathan; the PendingIntent was incorrectly marked as mutable. We
don't believe this posed any risk to any users as the app has very limited intent functionality.

**Conclusion:** Addressed in [PR #9886]

### 1.5.3.1 The app shall by default mask data in the User Interface when it is known to be sensitive

We agree with the assessment of Leviathan. Not masking this data in the UI enables shoulder surfing
attacks.

**Conclusion:** Addressed in [PR #9896]

### 1.6.2.1 The app only uses software components without known vulnerabilities

A transitive dependency was flagged as vulnerable but it was later discovered to be a false-positive.

**Conclusion:** False-positive

### 1.6.3.1 Compiler security features shall be enabled

The requested feature is not yet available in a stable version of Rust. There is an [ongoing issue]
that we are tracking internally to add support once available.

`libdatastore_shared_counter.so` comes from the preference library [datastore]. We've reported the
[issue upstream]. The library ([source code]) is very small; it is a wrapper for an atomic C++
counter. We've manually reviewed it and found no issues.

**Conclusion:** Finding retracted by Leviathan

### 1.8.2.1 The app shall be transparent about data collection and usage

When adding Google Play Payments in version 2023.8 we did not update our Google Play listing to
state that we store Purchase history. The Purchase history is linked to an account for a limited
time to be able to offer refunds. After 20 days the link is removed automatically.

**Conclusion:** Google Play listing was updated with Purchase history on February 24, 2026

### 1.8.3.1 Users shall have the ability to request their data to be deleted via an in-app mechanism

We had multiple discussions with Leviathan where we argued that account deletion makes little sense
in our use case. We already continuously remove user data, as described in our [privacy policy]. By
already applying these best practices and not storing personally identifiable information, the
value of account deletion is very limited. Adding it also opens up a potential vector for abuse and
irreversible accidental deletions. However, the requirements for MAP are clear: if a user can
create an account in the app, they should be able to delete it. From a user standpoint this also
allows them to more easily cut the link between the account and the purchase before the 20 days
have passed. Previously, users had to email support to sever this link.

**Conclusion:** Account deletion was added in [PR #9938]

## Summary

All the fixes were approved and verified by Leviathan on March 19 against [2026.3-beta3] which was
subsequently released as [2026.3] on March 23.

## Certificate and reports

Official results hosted by App Defense Alliance:
* [2026-04-10 App Directory entry](https://appdefensealliance.dev/directory?app=net.mullvad.mullvadvpn)
* [2026-04-10 Direct certificate link](https://appdefensealliance.dev/reports/net.mullvad.mullvadvpn_1775779200000000.pdf) (pending publication)

We also host the test reports (original and re-test of fixed findings) as well as the final compliance
report in our repository:
* [2026-02-23 MAP Developer Test Report v1](2026-02-23-leviathan-masa-map-developer-test-report-v1.pdf)
* [2026-04-16 MAP Developer Test Report v2](2026-04-16-leviathan-masa-map-developer-test-report-v2.pdf)
* [2026-04-16 MAP Compliance Report](2026-04-16-leviathan-masa-map-compliance-report.pdf)

## Last words

We would like to thank Leviathan for the thorough assessment. The communication was professional,
and the assessment was carried out to a high standard and provided us with valuable insights.

[Leviathan Security Group]: https://www.leviathansecurity.com/
[PR #9886]: https://github.com/mullvad/mullvadvpn-app/pull/9886
[PR #9896]: https://github.com/mullvad/mullvadvpn-app/pull/9896
[PR #9887]: https://github.com/mullvad/mullvadvpn-app/pull/9887
[ongoing issue]: https://github.com/rust-lang/rust/pull/146369
[datastore]: https://developer.android.com/topic/libraries/architecture/datastore
[issue upstream]: https://issuetracker.google.com/issues/487139126
[source code]: https://github.com/androidx/androidx/blob/androidx-main/datastore/datastore-core/src/androidMain/cpp/shared/shared_counter.cc
[privacy policy]: https://mullvad.net/en/help/privacy-policy
[PR #9938]: https://github.com/mullvad/mullvadvpn-app/pull/9938
[2026.3]: https://github.com/mullvad/mullvadvpn-app/releases/tag/android%2F2026.3
[2026.3-beta3]: https://github.com/mullvad/mullvadvpn-app/releases/tag/android%2F2026.3-beta3
