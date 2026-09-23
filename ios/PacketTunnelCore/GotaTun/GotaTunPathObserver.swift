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
    typealias PathUpdates = @Sendable () -> AsyncStream<Path>
    typealias AddressLookup = @Sendable (_ interface: String?) -> Set<String>

    private let clock: any Clock<Duration>
    private let makePathUpdates: PathUpdates
    private let localAddresses: AddressLookup
    private var observation: Task<Void, Never>?
    private var onChange: (@Sendable (Network.NWPath.Status) -> Void)?
    private var startedStatus: Network.NWPath.Status?
    private let logger = Logger(label: "GotaTunPathObserver")
    private var pendingLoss: Task<Void, Never>?
    private var pendingLossPath: Path?
    private var deliveredPath: Path?
    private var deliveredAddresses: Set<String> = []
    private var addressCheck: Task<Void, Never>?

    /// How long a reported loss of connectivity must persist before it is believed. Applying
    /// tunnel settings, and interface handoffs, momentarily leave the path unsatisfied.
    static let pathUpdateDebounceDelay: Duration = .milliseconds(250)

    /// How long after a path update the interface's addresses are compared.
    static let addressCheckDelay: Duration = .seconds(3)

    public init(clock: any Clock<Duration> = ContinuousClock()) {
        self.init(clock: clock, pathUpdates: Self.defaultPathUpdates, localAddresses: interfaceAddresses)
    }

    init(clock: any Clock<Duration>, pathUpdates: @escaping PathUpdates, localAddresses: @escaping AddressLookup) {
        self.clock = clock
        self.makePathUpdates = pathUpdates
        self.localAddresses = localAddresses
    }

    @discardableResult
    public func start(
        _ body: @escaping @Sendable (Network.NWPath.Status) -> Void
    ) async -> Network.NWPath.Status {
        if let startedStatus { return startedStatus }

        var iterator = makePathUpdates().makeAsyncIterator()
        let currentPath = await iterator.next().map(resolved)
        let currentStatus = currentPath?.status ?? .unsatisfied
        startedStatus = currentStatus
        deliveredPath = currentPath
        deliveredAddresses = localAddresses(currentPath?.interface)
        onChange = body

        observation = Task { [weak self] in
            while let path = await iterator.next() {
                await self?.handle(path)
            }
        }

        return currentStatus
    }

    public func stop() {
        // Cancelling ends iteration, which terminates the path updates.
        observation?.cancel()
        observation = nil
        onChange = nil
        cancelPendingLoss()
        addressCheck?.cancel()
        addressCheck = nil
    }

    private static func defaultPathUpdates() -> AsyncStream<Path> {
        let monitor = NWPathMonitor()
        return AsyncStream { continuation in
            monitor.pathUpdateHandler = { continuation.yield(Path($0)) }
            continuation.onTermination = { _ in monitor.cancel() }
            monitor.start(queue: DispatchQueue(label: "GotaTunPathObserver.monitor"))
        }
    }

    func handle(_ update: Path) {
        let path = resolved(update)

        // Losing a path should be debounced - .satisfied updates need not be debounced. This swallows spurious losses in
        // connectivity. Further losses replace the pending path but keep its deadline, so churn cannot postpone it.
        if path.status == .unsatisfied, pendingLoss != nil {
            pendingLossPath = path
            return
        }

        cancelPendingLoss()

        guard path != deliveredPath else {
            scheduleAddressCheck()
            return
        }

        guard path.status == .unsatisfied else {
            deliver(path)
            return
        }

        pendingLossPath = path
        pendingLoss = Task {
            try? await clock.sleep(for: Self.pathUpdateDebounceDelay)
            guard !Task.isCancelled, let pendingLossPath else { return }
            if pendingLossPath == deliveredPath {
                cancelPendingLoss()
            } else {
                deliver(pendingLossPath)
            }
        }
    }

    /// Does nothing while a check is pending, so frequent path updates cannot postpone it.
    private func scheduleAddressCheck() {
        guard addressCheck == nil, deliveredPath?.status == .satisfied else { return }

        addressCheck = Task {
            try? await clock.sleep(for: Self.addressCheckDelay)
            guard !Task.isCancelled else { return }
            addressCheck = nil

            guard let path = deliveredPath, path.status == .satisfied, pendingLoss == nil else { return }
            let addresses = localAddresses(path.interface)
            guard addresses != deliveredAddresses else { return }
            if addresses.isEmpty {
                handle(path)
            } else {
                deliver(path)
            }
        }
    }

    /// A satisfied path over an interface without addresses cannot carry traffic. This happens while an interface is
    /// torn down, so it is treated as a loss.
    private func resolved(_ path: Path) -> Path {
        guard path.status == .satisfied, localAddresses(path.interface).isEmpty else { return path }
        return Path(status: .unsatisfied, interface: path.interface, gateways: path.gateways)
    }

    private func cancelPendingLoss() {
        pendingLoss?.cancel()
        pendingLoss = nil
        pendingLossPath = nil
    }

    private func deliver(_ path: Path) {
        cancelPendingLoss()
        deliveredPath = path
        deliveredAddresses = localAddresses(path.interface)
        logger.debug(
            """
            Path changed: \(path.status), interface: \(path.interface ?? "none"), \
            gateways: \(path.gateways), addresses: \(deliveredAddresses.sorted())
            """
        )
        onChange?(path.status)
    }
}

extension GotaTunPathObserver {
    /// The parts of a path that matter to the tunnel's sockets: whether there is one, which interface
    /// the system prefers, and which network that interface is attached to.
    struct Path: Equatable, Sendable {
        let status: Network.NWPath.Status
        let interface: String?
        let gateways: [NWEndpoint]

        init(status: Network.NWPath.Status, interface: String?, gateways: [NWEndpoint] = []) {
            self.status = status
            self.interface = interface
            self.gateways = gateways
        }

        init(_ path: Network.NWPath) {
            self.init(status: path.status, interface: path.availableInterfaces.first?.name, gateways: path.gateways)
        }
    }
}

/// The IPv4 and IPv6 addresses assigned to `interface`.
private func interfaceAddresses(of interface: String?) -> Set<String> {
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
