//
//  RelayTests.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-11.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class RelayTests: BaseTestCase {
    override func setUp() {
        /*print("Adding UI interruption monitor")
        addUIInterruptionMonitor(withDescription: "System Alert") { (alert) -> Bool in
            if alert.buttons["Allow"].exists {
                alert.buttons["Allow"].tap()
                return true
            }

            return false
        }*/
    }

    func testAdBlockingViaRelay() throws {
        let app = XCUIApplication()
        app.launch()

        // allowAddVPNConfigurations()

        TermsOfServicePage(app)
            .tapAgree()

        Alert(app)
            .tapOkay()

        LoginPage(app)
            .tapAccountNumberTextField()
            .enterText(self.hasTimeAccountNumber)
            .tapAccountNumberSubmitButton()
            .verifySuccessIconShown()
            .verifyDeviceLabelShown()

        TunnelControlPage(app)
            .tapSelectLocationButton()

        SelectLocationPage(app)
            .tapLocationCellExpandButton(withName: "Sweden")
            .tapLocationCellExpandButton(withName: "Gothenburg")
            .tapLocationCell(withName: "se-got-wg-001")

        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        let alertAllowButton = springboard.buttons.element(boundBy: 0)
        if alertAllowButton.waitForExistence(timeout: 5) {
           alertAllowButton.tap()
        }

        sleep(1)
        springboard.typeText("123456")

        /*sleep(2)
        print("Swipe")
        app.otherElements.firstMatch.swipeLeft()
        sleep(2)
        print("Swipe")
        app.otherElements.firstMatch.swipeLeft()*/

        // allowAddVPNConfigurations()

        /*let request = URLRequest(url: URL(string: "http://www.mullvad.net")!)
        let task = URLSession.shared.dataTask(with: request) { (data, response, error) in
            if let error = error {
                print(error)
            }

            guard let httpResponse = response as? HTTPURLResponse else {
                print("No response received")
                return
            }

            guard let data = data else {
                print("No data received")
                return
            }

            print(httpResponse.statusCode)

            if let responseBody = String(data: data, encoding: .utf8) {
                print(responseBody)
            }
        }

        task.resume()*/

        sleep(10)
    }
}
