//
//  BackgroundTaskProvider.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

#if canImport(UIKit)

import Foundation
import UIKit

@available(iOSApplicationExtension, unavailable)
public protocol BackgroundTaskProviding: Sendable {
    var backgroundTimeRemaining: TimeInterval { get }
    nonisolated func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (@MainActor @Sendable () -> Void)?
    ) -> UIBackgroundTaskIdentifier

    func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier)
}

@available(iOSApplicationExtension, unavailable)
public final class BackgroundTaskProvider: BackgroundTaskProviding {
    nonisolated(unsafe) public var backgroundTimeRemaining: TimeInterval
    nonisolated(unsafe) weak var application: UIApplication!

    public init(backgroundTimeRemaining: TimeInterval, application: UIApplication) {
        self.backgroundTimeRemaining = backgroundTimeRemaining
        self.application = application
    }

    nonisolated public func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (@MainActor @Sendable () -> Void)? = nil
    ) -> UIBackgroundTaskIdentifier {
        application.beginBackgroundTask(withName: taskName, expirationHandler: handler)
    }

    public func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier) {
        application.endBackgroundTask(identifier)
    }
}
#endif
