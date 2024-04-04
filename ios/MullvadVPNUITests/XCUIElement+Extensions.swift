//
//  XCUIElement+Extensions.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import XCTest

extension XCUIElement {
    func waitForNonExistence(timeout: TimeInterval) -> Bool {
        let predicate = NSPredicate(format: "exists == FALSE")
        let expectation = XCTNSPredicateExpectation(predicate: predicate, object: self)

        _ = XCTWaiter().wait(for: [expectation], timeout: timeout)
        return !exists
    }
}
