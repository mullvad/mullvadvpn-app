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

    #if compiler(>=6)
    nonisolated
    public func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (@MainActor @Sendable () -> Void)?
    ) -> UIBackgroundTaskIdentifier {
        application.beginBackgroundTask(withName: taskName, expirationHandler: handler)
    }
    #else
    public func beginBackgroundTask(
        withName taskName: String?,
        expirationHandler handler: (() -> Void)?
    ) -> UIBackgroundTaskIdentifier {
        application.beginBackgroundTask(withName: taskName, expirationHandler: handler)
    }
    #endif

    public func endBackgroundTask(_ identifier: UIBackgroundTaskIdentifier) {
        application.endBackgroundTask(identifier)
    }
}
#endif
