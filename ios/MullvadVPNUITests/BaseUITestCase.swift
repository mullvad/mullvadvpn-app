//
//  BaseTestCase.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-01-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

class BaseUITestCase: XCTestCase {
    public static let defaultTimeout = 10.0

    // swiftlint:disable force_cast
    let noTimeAccountNumber = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["MullvadNoTimeAccountNumber"] as! String
    let hasTimeAccountNumber = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["MullvadHasTimeAccountNumber"] as! String
    let fiveWireGuardKeysAccountNumber = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["MullvadFiveWireGuardKeysAccountNumber"] as! String
    let iOSDevicePinCode = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["MullvadIOSDevicePinCode"] as! String
    let adServingDomain = Bundle(for: BaseUITestCase.self)
        .infoDictionary?["MullvadAdServingDomain"] as! String
    // swiftlint:enable force_cast

    /// Handle iOS add VPN configuration permission alert - allow and enter device PIN code
    func allowAddVPNConfigurations() {
        let springboard = XCUIApplication(bundleIdentifier: "com.apple.springboard")

        let alertAllowButton = springboard.buttons.element(boundBy: 0)
        if alertAllowButton.waitForExistence(timeout: Self.defaultTimeout) {
            alertAllowButton.tap()
        }

        _ = springboard.buttons["1"].waitForExistence(timeout: Self.defaultTimeout)
        springboard.typeText(iOSDevicePinCode)
    }
}
