//
//  SettingsTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-02-23.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class SettingsTests: LoggedOutUITestCase {
    func testSendProblemReport() throws {
        #if MULLVAD_ENVIRONMENT_STAGING
        let shouldSkipTest = false
        #else
        let shouldSkipTest = true
        #endif

        try XCTSkipIf(shouldSkipTest, "This test should only run in the staging environment")

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapReportAProblemCell()

        ProblemReportPage(app)
            .tapEmailTextField()
            .enterText("cookie@mullvad.net")
            .tapMessageTextView()
            .enterText("""
            Dear support
            This is a problem report from an iOS app test.
            """)
            .tapKeyboardDoneButton()
            .tapSendButton()

        ProblemReportSubmittedPage(app)
    }
}
