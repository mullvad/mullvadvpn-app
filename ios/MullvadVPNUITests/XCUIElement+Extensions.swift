//
//  XCUIElement+Extensions.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-03-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

extension XCUIElement {
    func scrollDownToElement(element: XCUIElement, maxScrolls: UInt = 5) {
        var count = 0
        while !element.isVisible && count < maxScrolls {
            swipeUp(velocity: .slow)
            count += 1
        }
    }

    func scrollUpToElement(element: XCUIElement, maxScrolls: UInt = 5) {
        var count = 0
        while !element.isVisible && count < maxScrolls {
            swipeDown(velocity: .slow)
            count += 1
        }
    }

    var isVisible: Bool {
        guard self.exists && !self.frame.isEmpty else { return false }
        return XCUIApplication().windows.element(boundBy: 0).frame.contains(self.frame)
    }

    @discardableResult
    func wait(
        for condition: Condition = .exists,
        timeout: Timeout = .short,
        hardAssertion: Bool = true,
        description: String? = nil
    ) -> Self {
        let exists = XCUIElement.wait(
            for: {
                switch condition {
                case .exists:
                    self.exists
                case .notExists:
                    !self.exists
                case .hittable:
                    self.isHittable
                }
            },
            timeout: timeout,
            description: description ?? "Waiting for existence"
        )

        if !exists && hardAssertion {
            XCTFail()
        }

        return self
    }

    @discardableResult
    func tapWhenHittable(timeout: Timeout = .medium, hardAssertion: Bool = true, description: String? = nil) -> Self {
        if wait(
            for: .hittable,
            timeout: timeout,
            hardAssertion: hardAssertion,
            description: description
        ).isHittable {
            self.tap()
        } else if hardAssertion {
            XCTFail()
        }

        return self
    }
}

// Borrowed and adapted from https://eng.wealthfront.com/2025/03/17/how-we-sped-up-ios-end-to-end-tests-by-over-50-with-40-lines-of-code/.
extension XCUIElement {
    enum Condition {
        case exists
        case notExists
        case hittable
    }

    enum Timeout: TimeInterval {
        case veryShort = 2
        case short = 4
        case medium = 8
        case long = 12
        case veryLong = 15
    }

    struct PredicatePollerDefaults {
        static let minPollInterval: TimeInterval = 0.2
        static let pollMultiplier: Double = 1.5
        static let maxPollInterval: TimeInterval = 2
        static let maxIterations: Int = 100
    }

    @discardableResult
    private static func wait(
        for condition: @escaping () -> Bool,
        timeout: Timeout = .medium,
        failureMessage: String = "Condition not met",
        description: String
    ) -> Bool {
        guard !condition() else {
            return true
        }

        let start = Date()
        let timeoutDate = start.addingTimeInterval(timeout.rawValue)
        let expectation = XCTestExpectation(description: description)
        var pollInterval = PredicatePollerDefaults.minPollInterval
        var iterationCount = 0

        while Date() < timeoutDate {
            iterationCount += 1

            let remainingTime = timeoutDate.timeIntervalSinceNow
            let effectivePollInterval = min(pollInterval, remainingTime)

            guard iterationCount <= PredicatePollerDefaults.maxIterations else {
                return false
            }

            guard !condition() else {
                expectation.fulfill()
                return true
            }

            guard effectivePollInterval > 0 else {
                break
            }

            RunLoop.current.run(until: Date().addingTimeInterval(effectivePollInterval))
            // Exponential backoff so CI doesn't get overwhelmed.
            pollInterval = min(
                max(
                    PredicatePollerDefaults.minPollInterval,
                    pollInterval * PredicatePollerDefaults.pollMultiplier
                ),
                PredicatePollerDefaults.maxPollInterval
            )
        }

        return false
    }
}
