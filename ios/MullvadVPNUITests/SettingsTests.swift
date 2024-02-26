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
        #if !MULLVAD_ENVIRONMENT_STAGING
        XCTFail("Only allowed against staging in order to avoid spamming support")
        return
        #endif

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
