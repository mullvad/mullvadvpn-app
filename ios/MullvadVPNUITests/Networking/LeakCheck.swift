//
//  LeakCheck.swift
//  MullvadVPN
//
//  Created by Niklas Berglund on 2024-12-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

class LeakCheck {
    static func assertNoLeaks(streams: [Stream], rules: [LeakRule]) {
        for rule in rules where rule.isViolated(streams: streams) {
            XCTFail("Leak rule violated")
        }
    }

    static func assertLeaks(streams: [Stream], rules: [LeakRule]) {
        for rule in rules where rule.isViolated(streams: streams) == false {
            XCTFail("Leak rule unexpectedly not violated when asserting leak")
        }
    }
}

protocol LeakRule {
    func isViolated(streams: [Stream]) -> Bool
}

class NoTrafficToHostLeakRule: LeakRule {
    let host: String

    init(host: String) {
        self.host = host
    }

    func isViolated(streams: [Stream]) -> Bool {
        streams.filter { $0.destinationAddress == host }.isEmpty == false
    }
}
