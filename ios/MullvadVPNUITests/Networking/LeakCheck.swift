//
//  LeakCheck.swift
//  MullvadVPN
//
//  Created by Niklas Berglund on 2024-12-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
