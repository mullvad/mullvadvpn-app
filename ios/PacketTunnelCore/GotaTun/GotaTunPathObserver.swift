// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import Network

/// A type providing default path observation for the GotaTun actor.
public protocol GotaTunPathObserverProtocol: Sendable {
    /// Start observing the default path and return its current status. Later changes are delivered
    /// to `body` in the order they occurred. Must be called once; further calls return the same
    /// status without restarting observation.
    ///
    /// A path update will be delivered even if reachability does not change - going from WiFi to modem or otherwise should still result in notifiying the user.
    @discardableResult
    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) async -> Network.NWPath.Status

    /// Stop observing the default path. The observer cannot be started again.
    func stop() async
}

public actor GotaTunPathObserver: GotaTunPathObserverProtocol {
    private let pathMonitor = NWPathMonitor()
    private var observation: Task<Void, Never>?
    private var startedStatus: Network.NWPath.Status?

    /// How long a reported loss of connectivity must persist before it is believed. Applying
    /// tunnel settings, and interface handoffs, momentarily leave the path unsatisfied.
    private static let pathUpdateDebounceDelay: Duration = .milliseconds(250)

    public init() {}

    @discardableResult
    public func start(
        _ body: @escaping @Sendable (Network.NWPath.Status) -> Void
    ) async -> Network.NWPath.Status {
        if let startedStatus { return startedStatus }

        var iterator = pathMonitor.makeAsyncIterator()
        let currentStatus = await iterator.next()?.status ?? .unsatisfied
        startedStatus = currentStatus

        observation = Task { [iterator] in
            var iterator = iterator
            var pendingLoss: Task<Void, Never>?
            defer { pendingLoss?.cancel() }

            while let status = await iterator.next()?.status {
                pendingLoss?.cancel()
                pendingLoss = nil

                // Losing a path should be debounced - .satisfied updates need not be debounced. This swallows spurious losses in connectivity.
                guard status == .unsatisfied else {
                    body(status)
                    continue
                }

                pendingLoss = Task {
                    try? await Task.sleep(for: Self.pathUpdateDebounceDelay)
                    guard !Task.isCancelled else { return }
                    body(status)
                }
            }
        }

        return currentStatus
    }

    public func stop() {
        // Cancelling ends the sequence, which ends `observation`.
        pathMonitor.cancel()
        observation = nil
    }
}
