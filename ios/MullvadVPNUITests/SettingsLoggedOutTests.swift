// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

class SettingsLoggedOutTests: LoggedOutUITestCase {

    private func fillInAndSendProblemReportForm() {
        ProblemReportPage(app)
            .tapEmailTextField()
            .enterText("cookie@mullvad.net")
            .tapMessageTextView()
            .enterText(
                """
                Dear support
                This is a problem report from an iOS app test.
                """
            )
            .dismissMessageEditingOverlay()
            .tapSendButton()
    }

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

        fillInAndSendProblemReportForm()

        ProblemReportSubmittedPage(app)
    }

    func testSendProblemReportFailure() throws {
        #if MULLVAD_ENVIRONMENT_STAGING
            let shouldSkipTest = false
        #else
            let shouldSkipTest = true
        #endif

        try XCTSkipIf(shouldSkipTest, "This test should only run in the staging environment")

        breakConnectivity()

        HeaderBar(app)
            .tapSettingsButton()

        SettingsPage(app)
            .tapReportAProblemCell()

        fillInAndSendProblemReportForm()

        ProblemReportFailurePage(app)
            .tapViewLogsButton()

        ProblemReportLogPage(app)
            .swipeDownToDismissModal()

        ProblemReportFailurePage(app)
            .tapEditMessageButton()

        ProblemReportPage(app)
    }
}

class SettingsLoggedInTests: LoggedInWithTimeUITestCase {
    func testLanguageSelection() throws {
        HeaderBar(app)
            .tapSettingsButton()

        TunnelControlPage(app)
            .tapConnectButton()
            .waitForConnectedLabel()

        SettingsPage(app)
            .tapLanguageCell()
            .dismissAlert()
            .tapDoneButton()

        TunnelControlPage(app)
            .tapDisconnectButton()
    }
}
