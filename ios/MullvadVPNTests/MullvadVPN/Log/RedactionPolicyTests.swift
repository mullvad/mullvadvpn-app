// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Testing

struct RedactionPolicyTests {
    let legacyLogRedaction = RedactionPolicy()

    @Test(
        "It should determine whether the log file is pre-new test log format",
        arguments: [
            (version: "2026.3", expect: true),
            (version: "2025.3-dev18", expect: true),
            (version: "2025.1", expect: true),
            (version: "2025.3-beta8", expect: true),
            (version: "2026.4", expect: false),
            (version: "2026.4-dev1", expect: false),
            (version: "2025.5-dev18", expect: true),
            (version: "2026.6", expect: false),
            (version: "2025.4-beta8", expect: true),
            (version: "2026.5-dev18", expect: false),
        ])
    func shouldRedact(version: String, expect: Bool) {
        let log = """
            MullvadVPN version \(version)
            [AppDelegate][debug] Registered app refresh task.
            [TunnelManager][debug] Refresh device state and tunnel status.
            """

        #expect(legacyLogRedaction.shouldRedact(content: log) == expect)
    }
}
