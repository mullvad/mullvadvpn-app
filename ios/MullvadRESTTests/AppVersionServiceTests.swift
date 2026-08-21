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

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings

class AppVersionServiceTests: XCTestCase {
    private let encoder = JSONEncoder()

    func testPerformVersionCheckNewVersionExists() async throws {
        let version = Double(Bundle.main.shortVersion)! + 1
        let bundleId = ApplicationTarget.mainApp.bundleIdentifier

        let mockData = try encoder.encode(
            [
                "results": [
                    [
                        "bundleId": bundleId,
                        "version": String(version),
                    ]
                ]
            ]
        )

        let appVersionService = AppVersionService(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            appPreferences: AppPreferences(),
            mainAppBundleIdentifier: bundleId

        )

        let shouldSendNotification = try await appVersionService.performVersionCheck()
        XCTAssertTrue(shouldSendNotification)
    }

    func testPerformVersionCheckNewVersionDoesNotExist() async throws {
        let bundleId = ApplicationTarget.mainApp.bundleIdentifier

        let mockData = try encoder.encode(
            [
                "results": [
                    [
                        "bundleId": bundleId,
                        "version": Bundle.main.shortVersion,
                    ]
                ]
            ]
        )

        let appVersionService = AppVersionService(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            appPreferences: AppPreferences(),
            mainAppBundleIdentifier: bundleId
        )

        let shouldSendNotification = try await appVersionService.performVersionCheck()
        XCTAssertFalse(shouldSendNotification)
    }

    func testVersionComparison() {
        XCTAssertTrue("2025.10".isNewerThan("2025.9"))
        XCTAssertTrue("2025.10".isNewerThan("2025.09"))

        XCTAssertFalse("2025.10".isNewerThan("2025.10"))
        XCTAssertFalse("2025.10".isNewerThan("2025.0100"))
    }
}
