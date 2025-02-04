//
//  UIApplication+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

@testable import MullvadTypes

struct UIApplicationStub: BackgroundTaskProviding {
    var backgroundTimeRemaining: TimeInterval { .infinity }

    func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier) {}

    #if compiler(>=6)
    func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (@MainActor @Sendable () -> Void)?
    )
        -> UIBackgroundTaskIdentifier {
        .invalid
    }
    #else
    func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (() -> Void)?
    ) -> UIBackgroundTaskIdentifier {
        .invalid
    }
    #endif
}
