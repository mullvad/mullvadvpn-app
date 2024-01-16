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
    // swiftlint:disable force_cast
    let noTimeAccountNumber = Bundle(for: AccountTests.self).infoDictionary?["MullvadNoTimeAccountNumber"] as! String
    let hasTimeAccountNumber = Bundle(for: AccountTests.self).infoDictionary?["MullvadHasTimeAccountNumber"] as! String
    let fiveWireGuardKeysAccountNumber = Bundle(for: AccountTests.self)
        .infoDictionary?["MullvadFiveWireGuardKeysAccountNumber"] as! String
    // swiftlint:enable force_cast

    func allowAddVPNConfigurations() {
        addUIInterruptionMonitor(withDescription: "System Alert") { (alert) -> Bool in
            let allowButton = alert.buttons["Allow"]
            allowButton.tap()
            return true
        }
    }
}
