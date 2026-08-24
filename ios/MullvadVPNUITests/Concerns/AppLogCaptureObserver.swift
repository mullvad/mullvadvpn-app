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

@MainActor
protocol AppLogConfigurable {
    var attachAppLogsOnFailure: Bool { get }
    var app: XCUIApplication { get }
    var target: MullvadExecutableTarget { get }
}

class AppLogCaptureObserver: NSObject, @preconcurrency XCTestObservation {

    @MainActor
    func testCaseDidFinish(_ testCase: XCTestCase) {
        guard
            let testRun = testCase.testRun,
            testRun.failureCount > 0,
            let configurable = testCase as? AppLogConfigurable,
            configurable.attachAppLogsOnFailure
        else {
            return
        }

        XCTContext.runActivity(named: "Record app logs") { activity in
            let app = configurable.app
            do {
                try app.relaunch(configurable.target)
            } catch {
                return
            }

            HeaderBar(app).tapSettingsButton()
            SettingsPage(app).tapReportAProblemCell()
            ProblemReportPage(app).tapViewAppLogsButton()

            let text = AppLogsPage(app).getAppLogText()

            let dateFormatter = DateFormatter()
            dateFormatter.dateFormat = "yyyy-MM-dd_HH-mm-ss"
            let dateString = dateFormatter.string(from: Date())
            let attachment = XCTAttachment(string: text)
            attachment.name = "app-log-\(dateString).log"
            activity.add(attachment)
        }
    }
}
