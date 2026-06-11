//
//  AppLogCaptureObserver.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
