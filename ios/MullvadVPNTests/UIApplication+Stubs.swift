//
//  UIApplication+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

@testable import MullvadTypes

struct UIApplicationStub: BackgroundTaskProvider {
    func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier) {}

    func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (() -> Void)?
    ) -> UIBackgroundTaskIdentifier {
        .invalid
    }
}
