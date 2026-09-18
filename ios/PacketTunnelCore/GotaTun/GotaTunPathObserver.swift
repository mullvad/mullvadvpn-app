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
import Logging
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
    private let logger = Logger(label: "GotaTunPathObserver")
    private var pendingLoss: Task<Void, Never>?
    private var deliveredRoute: Route?
    private var deliveredAddresses: Set<String> = []
    private var addressCheck: Task<Void, Never>?

    /// How long a reported loss of connectivity must persist before it is believed. Applying
    /// tunnel settings, and interface handoffs, momentarily leave the path unsatisfied.
    private static let pathUpdateDebounceDelay: Duration = .milliseconds(250)

    /// How long after a path update the interface's addresses are compared.
    private static let addressCheckDelay: Duration = .seconds(3)

    public init() {}

    @discardableResult
    public func start(
        _ body: @escaping @Sendable (Network.NWPath.Status) -> Void
    ) async -> Network.NWPath.Status {
        if let startedStatus { return startedStatus }

        var iterator = pathMonitor.makeAsyncIterator()
        let currentPath = await iterator.next()
        let currentStatus = currentPath?.status ?? .unsatisfied
        startedStatus = currentStatus
        deliveredRoute = currentPath.map(Route.init)
        deliveredAddresses = localAddresses(of: deliveredRoute?.interface)

        observation = Task { [weak self] in
            while let path = await iterator.next() {
                await self?.handle(path, body)
            }
        }

        return currentStatus
    }

    public func stop() {
        // Cancelling ends the sequence, which ends `observation`.
        pathMonitor.cancel()
        observation = nil
        pendingLoss?.cancel()
        pendingLoss = nil
        addressCheck?.cancel()
        addressCheck = nil
    }

    private func handle(_ path: Network.NWPath, _ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
        logger.debug("Received new path update: \(path), gateways: \(path.gateways)")
        pendingLoss?.cancel()
        pendingLoss = nil

        let route = Route(path)
        guard route != deliveredRoute else {
            scheduleAddressCheck(body)
            return
        }

        // Losing a path should be debounced - .satisfied updates need not be debounced. This swallows spurious losses in connectivity.
        guard path.status == .unsatisfied else {
            deliver(route, body)
            return
        }

        pendingLoss = Task {
            try? await Task.sleep(for: Self.pathUpdateDebounceDelay)
            guard !Task.isCancelled else { return }
            deliver(route, body)
        }
    }

    /// Does nothing while a check is pending, so frequent path updates cannot postpone it.
    private func scheduleAddressCheck(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
        guard addressCheck == nil, deliveredRoute?.status == .satisfied else { return }

        addressCheck = Task {
            try? await Task.sleep(for: Self.addressCheckDelay)
            guard !Task.isCancelled else { return }
            addressCheck = nil

            guard let route = deliveredRoute, route.status == .satisfied, pendingLoss == nil,
                localAddresses(of: route.interface) != deliveredAddresses
            else { return }
            deliver(route, body)
        }
    }

    private func deliver(_ route: Route, _ body: (Network.NWPath.Status) -> Void) {
        deliveredRoute = route
        deliveredAddresses = localAddresses(of: route.interface)
        logger.debug(
            """
            Path changed: \(path.status), interface: \(path.interface ?? "none"), \
            gateways: \(path.gateways), addresses: \(deliveredAddresses.sorted())
            """
        )
        body(route.status)
    }
}

/// The parts of a path that matter to the tunnel's sockets: whether there is one, which interface
/// the system prefers, and which network that interface is attached to.
private struct Route: Equatable {
    let status: Network.NWPath.Status
    let interface: String?
    let gateways: [NWEndpoint]

    init(_ path: Network.NWPath) {
        status = path.status
        interface = path.availableInterfaces.first?.name
        gateways = path.gateways
    }
}

/// The IPv4 and IPv6 addresses assigned to `interface`.
private func localAddresses(of interface: String?) -> Set<String> {
    var list: UnsafeMutablePointer<ifaddrs>?
    guard let interface, getifaddrs(&list) == 0, let first = list else { return [] }
    defer { freeifaddrs(list) }

    var addresses = Set<String>()
    for entry in sequence(first: first, next: { $0.pointee.ifa_next }) {
        guard String(cString: entry.pointee.ifa_name) == interface,
            let address = entry.pointee.ifa_addr,
            [AF_INET, AF_INET6].contains(Int32(address.pointee.sa_family))
        else { continue }

        var host = [CChar](repeating: 0, count: Int(NI_MAXHOST))
        guard
            getnameinfo(
                address, socklen_t(address.pointee.sa_len), &host, socklen_t(host.count), nil, 0, NI_NUMERICHOST
            ) == 0
        else { continue }
        addresses.insert(host.withUnsafeBufferPointer { String(cString: $0.baseAddress!) })
    }
    return addresses
}
