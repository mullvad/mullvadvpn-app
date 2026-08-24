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

    import MullvadTypes
    import UIKit

    @available(iOSApplicationExtension, unavailable)
    public final class BackgroundObserver: OperationObserver {
        public let name: String
        public let backgroundTaskProvider: BackgroundTaskProviding
        public let cancelUponExpiration: Bool

        private var taskIdentifier: UIBackgroundTaskIdentifier?

        public init(backgroundTaskProvider: BackgroundTaskProviding, name: String, cancelUponExpiration: Bool) {
            self.backgroundTaskProvider = backgroundTaskProvider
            self.name = name
            self.cancelUponExpiration = cancelUponExpiration
        }

        public func didAttach(to operation: Operation) {
            let expirationHandler =
                cancelUponExpiration
                ? { @MainActor in operation.cancel() } as? @MainActor @Sendable () -> Void
                : nil

            taskIdentifier = backgroundTaskProvider.beginBackgroundTask(
                withName: name,
                expirationHandler: expirationHandler
            )
        }

        public func operationDidStart(_ operation: Operation) {
            // no-op
        }

        public func operationDidCancel(_ operation: Operation) {
            // no-op
        }

        public func operationDidFinish(_ operation: Operation, error: Error?) {
            if let taskIdentifier {
                backgroundTaskProvider.endBackgroundTask(taskIdentifier)
            }
        }
    }

#endif
