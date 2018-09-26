# 2018-09-24 - Assured and Cure53

Eight testers from [Cure53](https://cure53.de/) and [Assured](https://assured.se/) spent a total of
18 days to complete the audit of version [2018.2] of the Mullvad VPN app.

As summarized in the report, "the assessment yielded a total of seven issues, which [is] an
exceptionally small number given the complex field of the VPN software and the connected, vast
attack surface."

Of those seven, six issues related to the app, none of which were remotely exploitable. In addition,
the testers found no traffic leaks and no ways for a network-based attacker to force leaks. The
remaining issue had to do with our website.

More information about the audit, and our comments on the issues in the report, can be found on the
Mullvad blog:

* [2018-09-20 - An audit has taken place](https://mullvad.net/en/blog/2018/9/20/security-audit-mullvad-app-completed-please-upgrade/)
* [2018-09-24 - The full reports, and our comments](https://mullvad.net/en/blog/2018/9/24/read-results-security-audit-mullvad-app/)

## Read the report

The final report is available
[on Cure53's website](https://cure53.de/pentest-report_mullvad_v2.pdf).

Also public is the [initial report](https://cure53.de/pentest-report_mullvad_v1.pdf) which is the
version that was initially presented to us. After a discussion with the auditors about the use of
certain terminology, they adjusted the report to provide better clarity and produced the final
version.

The reports are also available directly in this repository:
* [pentest-report_mullvad_v2.pdf](./pentest-report_mullvad_v2.pdf)
* [pentest-report_mullvad_v1.pdf](./pentest-report_mullvad_v1.pdf)

## Overview of findings

Of the seven issues found, the two identified vulnerabilities required local access to the computer.
Of the five miscellaneous issues, three required local access, one pertained to our website, and the
last one reflected on software dependencies.

Regarding the five findings that depended on local access, it should be noted that in general we do
not consider attackers with local access to be part of our threat model. Nonetheless, we will of
course consider all recommendations made by the auditors to further improve the security of our app.

Please feel free to contact us if you have any questions after reading this post or the audit
report.

### Identified vulnerabilities

* __MUL-01-004 Windows__: Privilege escalation by replacing executables (Critical)

  _Our comment_: Solved in app version [2018.3]. Under certain conditions, a user with local access
  could abuse the app to gain administrative privileges.

* __MUL-01-006 Daemon__: Any user can issue WebSocket commands (High)

  _Our comment_: Any user with local access can control the app. This is currently intentional, but
  we will consider the auditors' recommendations. It should also be noted that we replaced WebSocket
  with IPC.


### Miscellaneous issues

As described by the auditors, "This section covers those noteworthy findings that did not lead to an
exploit but might aid an attacker in achieving their malicious goals in the future.

"Most of these results are vulnerable code snippets that did not provide an easy way to be called.
Conclusively, while a vulnerability is present, an exploit might not always be possible."

* __MUL-01-001 App__: Missing Browser Window preferences allow RCE (Info)

  _Our comment_: Requires a local user to drag a malicious file onto the app window. We are looking
  into this.

* __MUL-01-002 App__: WebSocket leaks real IP addresses and geolocation (Medium)

  _Our comment_: By its current design, all local users should be able to query the app for current
  status and information. See also MUL-01-006. We are looking into this.

* __MUL-01-003 Daemon__: Weak permissions on config and log files (Low)

  _Our comment_: A local user can read the configuration and log files of the app. We are looking
  into this.

* __MUL-01-005 OOS__: CSRF on adding and removing forwarded ports (Low)

  _Our comment_: Fixed on 20 September 2018.

* __MUL-01-007 App__: Lax version requirements for Node dependencies (Info)

  _Our comment_: We are looking into this.


[2018.2]: ../CHANGELOG.md#20182---2018-08-13
[2018.3]: ../CHANGELOG.md#20183---2018-09-17
