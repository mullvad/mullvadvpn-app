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
        let conditionMet = XCUIElement.wait(
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

        if !conditionMet && failOnUnmetCondition {
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
        // API calls with `RetryStrategy.default` have a maximum timeout of 36 seconds
        // 10 sec * 3 tries + 2 sec + 2 sec ^ 2 = 36
        case longerThanMullvadAPITimeout = 37
        case extremelyLong = 180

        var pollInterval: TimeInterval {
            switch self {
            case .short, .default, .long, .veryLong: 0.2
            case .longerThanMullvadAPITimeout: 0.5
            case .extremelyLong: 1
            }
        }

        var maxIterations: Int {
            switch self {
            case .short, .default, .long, .veryLong, .longerThanMullvadAPITimeout: 100
            // 1 check per second for 180 seconds < 200
            case .extremelyLong: 200
            }
        }
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
            if iterationCount > timeout.maxIterations {
                return false
            }

            if condition() {
                expectation.fulfill()
                return true
            }

            RunLoop.current.run(until: Date().addingTimeInterval(timeout.pollInterval))
        }

        return false
    }
}

extension XCUIElement {
    @available(*, deprecated, message: "Use wait(for:timeout:failOnUnmetCondition:description)")
    func waitForExistence(timeout: TimeInterval) -> Bool {
        existsAfterWait(timeout: Timeout(rawValue: timeout) ?? .default)
    }

    @available(*, deprecated, message: "Use wait(for:timeout:failOnUnmetCondition:description)")
    func waitForNonExistence(timeout: TimeInterval) -> Bool {
        notExistsAfterWait(timeout: Timeout(rawValue: timeout) ?? .default)
    }

    @available(*, deprecated, message: "Use wait(for:timeout:failOnUnmetCondition:description)")
    func wait<V>(for keyPath: KeyPath<XCUIElement, V>, toEqual expectedValue: V, timeout: TimeInterval) -> Bool
    where V: Equatable {
        let timeout = Timeout(rawValue: timeout) ?? .default

        do {
            return switch keyPath {
            case \.exists:
                if try XCTUnwrap(expectedValue as? Bool) == true {
                    existsAfterWait(timeout: timeout)
                } else {
                    notExistsAfterWait(timeout: timeout)
                }
            case \.isHittable:
                wait(for: .hittable, timeout: timeout).isHittable
            default:
                throw NSError()
            }
        } catch {
            XCTFail("Could not map KeyPath to Condition")
        }

        return false
    }
}

extension XCUIElement {
    func clearText(app: XCUIApplication) {
        tapWhenHittable()
        press(forDuration: 1.0)

        let selectAll = app.menuItems["Select All"]
        if selectAll.existsAfterWait() {
            selectAll.tap()
            typeText(XCUIKeyboardKey.delete.rawValue)
        }
    }
}
