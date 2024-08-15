//
//  UIApplication+Extensions.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

#if canImport(UIKit)

import Foundation
import UIKit

public protocol BackgroundTaskProvider {
    var backgroundTimeRemaining: TimeInterval { get }
    func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier)

    #if compiler(>=6)
    nonisolated
    func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (@MainActor @Sendable () -> Void)?
    ) -> UIBackgroundTaskIdentifier
    #else
    func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (() -> Void)?
    ) -> UIBackgroundTaskIdentifier
    #endif
}

extension UIApplication: BackgroundTaskProvider {}

#endif
