//
//  AppStoreMetaDataServiceTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-01-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings

class AppStoreMetaDataServiceTests: XCTestCase {
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

        let metaDataService = AppStoreMetaDataService(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            appPreferences: AppPreferences(),
            mainAppBundleIdentifier: bundleId

        )

        let shouldSendNotification = try await metaDataService.performVersionCheck()
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

        let metaDataService = AppStoreMetaDataService(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            appPreferences: AppPreferences(),
            mainAppBundleIdentifier: bundleId
        )

        let shouldSendNotification = try await metaDataService.performVersionCheck()
        XCTAssertFalse(shouldSendNotification)
    }

    func testVersionComparison() {
        XCTAssertTrue("2025.10".isNewerThan("2025.9"))
        XCTAssertTrue("2025.10".isNewerThan("2025.09"))

        XCTAssertFalse("2025.10".isNewerThan("2025.10"))
        XCTAssertFalse("2025.10".isNewerThan("2025.0100"))
    }
}
