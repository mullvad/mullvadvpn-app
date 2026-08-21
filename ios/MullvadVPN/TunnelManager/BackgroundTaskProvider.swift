// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
