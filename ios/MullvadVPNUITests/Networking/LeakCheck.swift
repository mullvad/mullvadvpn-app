// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

class LeakCheck {
    static func assertNoLeaks(streams: [Stream], rules: [NoTrafficToHostLeakRule]) {
        XCTAssertFalse(streams.isEmpty, "No streams to leak check")
        XCTAssertFalse(rules.isEmpty, "No leak rules to check")

        for rule in rules where rule.isViolated(streams: streams) {
            XCTFail("Leaked traffic destined to \(rule.host) outside of the tunnel connection")
        }
    }

    static func assertLeaks(streams: [Stream], rules: [NoTrafficToHostLeakRule]) {
        XCTAssertFalse(streams.isEmpty, "No streams to leak check")
        XCTAssertFalse(rules.isEmpty, "No leak rules to check")

        for rule in rules where rule.isViolated(streams: streams) == false {
            XCTFail("Expected to leak traffic to \(rule.host) outside of tunnel")
        }
    }
}

class NoTrafficToHostLeakRule {
    let host: String

    init(host: String) {
        self.host = host
    }

    func isViolated(streams: [Stream]) -> Bool {
        streams.filter { $0.destinationAddress == host }.isEmpty == false
    }
}
