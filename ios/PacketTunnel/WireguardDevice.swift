//
//  WireguardDevice.swift
//  PacketTunnel
//
//  Created by pronebird on 16/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import os

/// A class describing the `wireguard-go` interactions
///
/// - Thread safety:
/// This class is thread safe.
class WireguardDevice {

    typealias WireguardLogHandler = (WireguardLogLevel, String) -> Void

    /// An error type describing the errors returned by `WireguardDevice`
    enum Error: ChainedError {
        /// A failure to obtain the tunnel device file descriptor
        case cannotLocateSocketDescriptor

        /// A failure to start the Wireguard backend
        case start(Int32)

        /// A failure that indicates that Wireguard has not been started yet
        case notStarted

        /// A failure that indicates that Wireguard has already been started
        case alreadyStarted

        /// A failure to resolve an endpoint
        case resolveEndpoint(AnyIPEndpoint, Swift.Error)

        var errorDescription: String? {
            switch self {
            case .cannotLocateSocketDescriptor:
                return "Unable to locate the file descriptor for socket."
            case .start(let wgErrorCode):
                return "Failed to start Wireguard. Return code: \(wgErrorCode)"
            case .notStarted:
                return "Wireguard has not been started yet"
            case .alreadyStarted:
                return "Wireguard has already been started"
            case .resolveEndpoint(let endpoint, _):
                return "Failed to resolve the endpoint: \(endpoint)"
            }
        }
    }

    /// A global Wireguard log handler
    /// It should only be accessed from the `loggingQueue`
    private static var wireguardLogHandler: WireguardLogHandler?

    /// A private queue used for Wireguard logging
    private static let loggingQueue = DispatchQueue(
        label: "net.mullvad.vpn.packet-tunnel.wireguard-device.global-logging-queue",
        qos: .background
    )

    /// A private queue used to synchronize access to `WireguardDevice` members
    private let workQueue = DispatchQueue(
        label: "net.mullvad.vpn.packet-tunnel.wireguard-device.work-queue"
    )

    /// A private queue used for network monitor
    private let networkMonitorQueue = DispatchQueue(
        label: "net.mullvad.vpn.packet-tunnel.network-monitor"
    )

    /// Network routes monitor
    private var networkMonitor: NWPathMonitor?

    /// A tunnel device descriptor
    private let tunFd: Int32

    /// A wireguard internal handle returned by `wgTurnOn` that's used to associate the calls
    /// with the specific Wireguard tunnel.
    private var wireguardHandle: Int32?

    /// Active configuration
    private var configuration: WireguardConfiguration?

    /// Returns a Wireguard version
    class var version: String {
        String(cString: wgVersion())
    }

    /// Set global Wireguard log handler.
    /// The given handler is dispatched on a background serial queue.
    ///
    /// - Thread safety:
    /// This function is thread safe
    class func setLogger(with handler: @escaping WireguardLogHandler) {
        WireguardDevice.loggingQueue.async {
            WireguardDevice.wireguardLogHandler = handler
        }

        wgSetLogger { (level, messagePtr) in
            guard let message = messagePtr.map({ String(cString: $0) }) else { return }
            let logType = WireguardLogLevel(rawValue: level) ?? .debug

            WireguardDevice.loggingQueue.async {
                WireguardDevice.wireguardLogHandler?(logType, message)
            }
        }
    }

    // MARK: - Initialization

    /// A designated initializer
    class func fromPacketFlow(_ packetFlow: NEPacketTunnelFlow) -> Result<WireguardDevice, Error> {
        if let fd = packetFlow.value(forKeyPath: "socket.fileDescriptor") as? Int32 {
            return .success(.init(tunFd: fd))
        } else {
            return .failure(.cannotLocateSocketDescriptor)
        }
    }

    /// Private initializer
    private init(tunFd: Int32) {
        self.tunFd = tunFd
    }

    deinit {
        networkMonitor?.cancel()
    }

    // MARK: - Public methods

    func start(configuration: WireguardConfiguration, completionHandler: @escaping (Result<(), Error>) -> Void) {
        workQueue.async {
            guard self.wireguardHandle == nil else {
                completionHandler(.failure(.alreadyStarted))
                return
            }

            let resolvedConfiguration = Self.resolveConfiguration(configuration)
            let handle = resolvedConfiguration
                .uapiConfiguration()
                .toRawWireguardConfigString()
                .withCString { wgTurnOn($0, self.tunFd) }

            if handle >= 0 {
                self.wireguardHandle = handle
                self.configuration = configuration

                self.startNetworkMonitor()

                completionHandler(.success(()))
            } else {
                completionHandler(.failure(.start(handle)))
            }
        }
    }

    func stop(completionHandler: @escaping (Result<(), Error>) -> Void) {
        workQueue.async {
            if let handle = self.wireguardHandle {
                self.networkMonitor?.cancel()
                self.networkMonitor = nil

                wgTurnOff(handle)
                self.wireguardHandle = nil

                completionHandler(.success(()))
            } else {
                completionHandler(.failure(.notStarted))
            }
        }
    }

    func setConfiguration(_ newConfiguration: WireguardConfiguration, completionHandler: @escaping (Result<(), Error>) -> Void) {
        workQueue.async {
            if let handle = self.wireguardHandle {
                let resolvedConfiguration = Self.resolveConfiguration(newConfiguration)
                let commands = resolvedConfiguration.uapiConfiguration()

                Self.setWireguardConfig(handle: handle, commands: commands)

                self.configuration = newConfiguration

                completionHandler(.success(()))
            } else {
                completionHandler(.failure(.notStarted))
            }
        }
    }

    func getInterfaceName() -> String? {
        var buffer = [UInt8](repeating: 0, count: Int(IFNAMSIZ))

        return buffer.withUnsafeMutableBufferPointer { (mutableBufferPointer) in
            guard let baseAddress = mutableBufferPointer.baseAddress else { return nil }

            var ifnameSize = socklen_t(IFNAMSIZ)
            let result = getsockopt(
                self.tunFd,
                2 /* SYSPROTO_CONTROL */,
                2 /* UTUN_OPT_IFNAME */,
                baseAddress,
                &ifnameSize)

            if result == 0 {
                return String(cString: baseAddress)
            } else {
                return nil
            }
        }
    }

    // MARK: - Private methods

    private class func setWireguardConfig(handle: Int32, commands: [WireguardCommand]) {
        // Ignore empty payloads
        guard !commands.isEmpty else { return }

        _ = commands.toRawWireguardConfigString()
            .withCString { wgSetConfig(handle, $0) }
    }

    private class func resolveConfiguration(_ configuration: WireguardConfiguration)
        -> WireguardConfiguration
    {
        return WireguardConfiguration(
            privateKey: configuration.privateKey,
            peers: resolvePeers(configuration.peers),
            allowedIPs: configuration.allowedIPs
        )
    }

    private class func resolvePeers(_ peers: [WireguardPeer]) -> [WireguardPeer] {
        var newPeers = [WireguardPeer]()

        for peer in peers {
            switch self.resolvePeer(peer) {
            case .success(let resolvedPeer):
                newPeers.append(resolvedPeer)
            case .failure(_):
                // Fix me: Ignore resolution error and carry on with the last known peer
                newPeers.append(peer)
            }
        }

        return newPeers
    }

    private class func resolvePeer(_ peer: WireguardPeer) -> Result<WireguardPeer, Error> {
        switch peer.withReresolvedEndpoint() {
        case .success(let resolvedPeer):
            if "\(peer.endpoint.ip)" == "\(resolvedPeer.endpoint.ip)" {
                os_log(.debug, log: wireguardDeviceLog,
                       "DNS64: mapped %{public}s to itself", "\(resolvedPeer.endpoint.ip)")
            } else {
                os_log(.debug, log: wireguardDeviceLog,
                       "DNS64: mapped %{public}s to %{public}s",
                       "\(peer.endpoint.ip)", "\(resolvedPeer.endpoint.ip)")
            }

            return .success(resolvedPeer)

        case .failure(let error):
            os_log(.error, log: wireguardDeviceLog,
                   "Failed to re-resolve the peer: %{public}s. Error: %{public}s",
                   "\(peer.endpoint.ip)", error.localizedDescription)

            return .failure(.resolveEndpoint(peer.endpoint, error))
        }
    }

    // MARK: - Network monitoring

    private func startNetworkMonitor() {
        self.networkMonitor?.cancel()

        let networkMonitor = NWPathMonitor()
        networkMonitor.pathUpdateHandler = { [weak self] (path) in
            self?.didReceiveNetworkPathUpdate(path: path)
        }
        networkMonitor.start(queue: networkMonitorQueue)
        self.networkMonitor = networkMonitor
    }

    private func didReceiveNetworkPathUpdate(path: Network.NWPath) {
        workQueue.async {
            guard let handle = self.wireguardHandle else { return }

            os_log(.debug, log: wireguardDeviceLog,
                   "Network change detected. Status: %{public}s, interfaces %{public}s.",
                   String(describing: path.status),
                   String(describing: path.availableInterfaces))

            // Re-resolve endpoints on network changes
            if let currentConfiguration = self.configuration {
                let resolvedConfiguration = Self.resolveConfiguration(currentConfiguration)
                let commands = resolvedConfiguration.endpointUapiConfiguration()

                Self.setWireguardConfig(handle: handle, commands: commands)
            }

            // Tell Wireguard to re-open sockets and bind them to the new network interface
            wgBumpSockets(handle)
        }
    }
}

/// A enum describing Wireguard log levels defined in `api-ios.go` from `wireguard-apple` repository
enum WireguardLogLevel: Int32 {
    case debug = 0
    case info = 1
    case error = 2

    var osLogType: OSLogType {
        switch self {
        case .debug:
            return .debug
        case .info:
            return .info
        case .error:
            return .error
        }
    }
}

extension Network.NWPath.Status: CustomDebugStringConvertible {
    public var debugDescription: String {
        var output = "NWPath.Status."

        switch self {
        case .requiresConnection:
            output += "requiresConnection"
        case .satisfied:
            output += "satisfied"
        case .unsatisfied:
            output += "unsatisfied"
        @unknown default:
            output += "unknown"
        }

        return output
    }
}
