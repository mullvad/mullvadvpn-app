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

    /// Waits for element to exist and returns true if it does so within the specified time frame.
    /// - Parameters:
    ///     - timeout: Waiting time. Defaults to `Timeout.default`.
    ///     - description: String describing the reason for waiting.
    func existsAfterWait(
        timeout: Timeout = .default,
        description: String? = nil
    ) -> Bool {
        wait(
            for: .exists,
            timeout: timeout,
            failOnUnmetCondition: false,
            description: description
        ).exists
    }

    /// Waits for element to not exist and returns true if it doesn't within the specified time frame.
    /// - Parameters:
    ///     - timeout: Waiting time. Defaults to `Timeout.default`.
    ///     - description: String describing the reason for waiting.
    func notExistsAfterWait(
        timeout: Timeout = .default,
        description: String? = nil
    ) -> Bool {
        !wait(
            for: .notExists,
            timeout: timeout,
            failOnUnmetCondition: false,
            description: description
        ).exists
    }

    /// Waits for element to meet a certain condition within the specified time frame.
    /// - Parameters:
    ///     - condition: The condition to wait for. Defaults to `Condition.exists`.
    ///     - timeout: Waiting time. Defaults to `Timeout.default`.
    ///     - failOnUnmetCondition: If true, fails the test if the condition is not met.
    ///     - description: String describing the reason for waiting.
    /// - Note: It's preferred to use `existsAfterWait()`, `notExistsAfterWait()` or `tapWhenHittable()`
    /// to handle those respective specific scenarios.
    @discardableResult
    func wait(
        for condition: Condition = .exists,
        timeout: Timeout = .default,
        failOnUnmetCondition: Bool = true,
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
            description: description
        )

        if !exists && failOnUnmetCondition {
            XCTFail(description ?? "Element failed to meet condition '\(condition)'")
        }

        return self
    }

    /// Waits for element to be hittable and, if successful, taps it.
    /// - Parameters:
    ///     - timeout: Waiting time. Defaults to `Timeout.default`.
    ///     - failOnUnmetCondition: If true, fails the test if the condition is not met.
    ///     - description: String describing the reason for waiting.
    @discardableResult
    func tapWhenHittable(
        timeout: Timeout = .default,
        failOnUnmetCondition: Bool = true,
        description: String? = nil
    ) -> Self {
        if wait(
            for: .hittable,
            timeout: timeout,
            failOnUnmetCondition: failOnUnmetCondition,
            description: description
        ).isHittable {
            tap()
        } else if failOnUnmetCondition {
            XCTFail(description ?? "Failed to tap element after timeout")
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
        case short = 1
        case `default` = 5
        case long = 15
        case veryLong = 20
        case extremelyLong = 180
    }

    private struct PredicatePollerDefaults {
        static let pollInterval: TimeInterval = 0.2
        static let maxIterations: Int = 100
    }

    // This function actively polls the hierarchy on a set interval. This speeds up the waiting process
    // siginificantly by returning much sooner than the default system `waitForExistence()` function.
    @discardableResult
    private static func wait(
        for condition: @escaping () -> Bool,
        timeout: Timeout = .default,
        failureMessage: String = "Condition not met",
        description: String? = nil
    ) -> Bool {
        if condition() {
            return true
        }

        let timeoutDate = Date().addingTimeInterval(timeout.rawValue)
        let expectation = XCTestExpectation(description: description ?? "Waiting for condition to be met")
        var iterationCount = 0

        while Date() < timeoutDate {
            iterationCount += 1
            if iterationCount > PredicatePollerDefaults.maxIterations {
                return false
            }

            if condition() {
                expectation.fulfill()
                return true
            }

            RunLoop.current.run(until: Date().addingTimeInterval(PredicatePollerDefaults.pollInterval))
        }

        return false
    }

    @available(*, deprecated, message: "Use wait(for:timeout:failOnUnmetCondition:description)")
    func waitForExistence(timeout: TimeInterval) -> Bool {
        existsAfterWait(timeout: Timeout(rawValue: timeout) ?? .default)
    }

    @available(*, deprecated, message: "Use wait(for:timeout:failOnUnmetCondition:description)")
    func waitForNonExistence(timeout: TimeInterval) -> Bool {
        notExistsAfterWait(timeout: Timeout(rawValue: timeout) ?? .default)
    }
}
