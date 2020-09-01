//
//  WireguardDevice.swift
//  PacketTunnel
//
//  Created by pronebird on 16/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import Logging

/// A class describing the `wireguard-go` interactions
///
/// - Thread safety:
/// This class is thread safe.
class WireguardDevice {

    /// An error type describing the errors returned by `WireguardDevice`
    enum Error: ChainedError {
        /// A failure to obtain the tunnel device file descriptor
        case cannotLocateSocketDescriptor

        /// A failure to duplicate the socket descriptor.
        /// The associated value contains the `errno` from a syscall to `dup`
        case cannotDuplicateSocketDescriptor(Int32)

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
                return "Cannot locate the socket file descriptor."
            case .cannotDuplicateSocketDescriptor(let posixErrorCode):
                return "Cannot duplicate the socket file descriptor. Errno: \(posixErrorCode)"
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

    /// A global Wireguard logger
    /// It should only be accessed from the `loggingQueue`
    private static var tunnelLogger: Logger?

    /// A logger used by WireguardDevice
    private let logger = Logger(label: "WireguardDevice")

    /// A private queue used for Wireguard logging
    private static let loggingQueue = DispatchQueue(
        label: "net.mullvad.vpn.packet-tunnel.wireguard-device.global-logging-queue",
        qos: .background
    )

    /// A private queue used to synchronize access to `WireguardDevice` members
    private let workQueue = DispatchQueue(
        label: "net.mullvad.vpn.packet-tunnel.wireguard-device.work-queue"
    )

    /// Network routes monitor
    private var networkMonitor: NWPathMonitor?

    /// A tunnel device source socket file descriptor
    private let tunnelFileDescriptor: Int32

    /// A wireguard internal handle returned by `wgTurnOn` that's used to associate the calls
    /// with the specific Wireguard tunnel.
    private var wireguardHandle: Int32?

    /// Active configuration
    private var configuration: WireguardConfiguration?

    /// A flag that indicates that the device has started
    private var isStarted = false

    /// A flag that indicates whether the last known network path was satisfied
    private var isPathSatisfied = true

    /// Returns a Wireguard version
    class var version: String {
        String(cString: wgVersion())
    }

    /// Set global Wireguard log handler.
    /// The given handler is dispatched on a background serial queue.
    ///
    /// - Thread safety:
    /// This function is thread safe
    class func setTunnelLogger(_ logger: Logger) {
        WireguardDevice.loggingQueue.async {
            WireguardDevice.tunnelLogger = logger
        }

        wgSetLogger { (level, messagePtr) in
            guard let message = messagePtr.map({ String(cString: $0) })?
                .trimmingCharacters(in: .newlines) else { return }
            let logLevel = WireguardLogLevel(rawValue: level) ?? .debug

            WireguardDevice.loggingQueue.async {
                WireguardDevice.tunnelLogger?.log(level: logLevel.loggerLevel, Logger.Message(stringLiteral: message))
            }
        }
    }

    // MARK: - Initialization

    /// A designated initializer
    class func fromPacketFlow(_ packetFlow: NEPacketTunnelFlow) -> Result<WireguardDevice, Error> {
        if let fd = packetFlow.value(forKeyPath: "socket.fileDescriptor") as? Int32 {
            return .success(.init(tunnelFileDescriptor: fd))
        } else {
            return .failure(.cannotLocateSocketDescriptor)
        }
    }

    /// Private initializer
    private init(tunnelFileDescriptor: Int32) {
        self.tunnelFileDescriptor = tunnelFileDescriptor
    }

    deinit {
        networkMonitor?.cancel()
        stopWireguardBackend()
    }

    // MARK: - Public methods

    func start(queue: DispatchQueue?, configuration: WireguardConfiguration, completionHandler: @escaping (Result<(), Error>) -> Void) {
        workQueue.async {
            guard !self.isStarted else {
                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.failure(.alreadyStarted))
                }
                return
            }

            assert(self.wireguardHandle == nil)

            let resolvedConfiguration = self.resolveConfiguration(configuration)

            switch self.startWireguardBackend(resolvedConfiguration: resolvedConfiguration) {
            case .success:
                self.isStarted = true
                self.isPathSatisfied = true
                self.configuration = configuration

                self.startNetworkMonitor()

                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.success(()))
                }

            case .failure(let error):
                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.failure(error))
                }
            }
        }
    }

    func stop(queue: DispatchQueue?, completionHandler: @escaping (Result<(), Error>) -> Void) {
        workQueue.async {
            if self.isStarted {
                self.networkMonitor?.cancel()
                self.networkMonitor = nil

                self.stopWireguardBackend()
                self.isStarted = false

                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.success(()))
                }
            } else {
                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.failure(.notStarted))
                }
            }
        }
    }

    func setConfiguration(_ newConfiguration: WireguardConfiguration, queue: DispatchQueue?, completionHandler: @escaping (Result<(), Error>) -> Void) {
        workQueue.async {
            if self.isStarted {
                if let handle = self.wireguardHandle {
                    let resolvedConfiguration = self.resolveConfiguration(newConfiguration)
                    let commands = resolvedConfiguration.uapiConfiguration()

                    Self.setWireguardConfig(handle: handle, commands: commands)
                }

                self.configuration = newConfiguration

                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.success(()))
                }
            } else {
                queue.performOnWrappedOrCurrentQueue {
                    completionHandler(.failure(.notStarted))
                }
            }
        }
    }

    func getInterfaceName() -> String? {
        var buffer = [UInt8](repeating: 0, count: Int(IFNAMSIZ))

        return buffer.withUnsafeMutableBufferPointer { (mutableBufferPointer) in
            guard let baseAddress = mutableBufferPointer.baseAddress else { return nil }

            var ifnameSize = socklen_t(IFNAMSIZ)
            let result = getsockopt(
                self.tunnelFileDescriptor,
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

    private func startWireguardBackend(resolvedConfiguration: WireguardConfiguration) -> Result<(), Error> {
        assert(self.wireguardHandle == nil)

        // Duplicate the tunnel file descriptor to prevent `wgTurnOff` from closing it
        let duplicateFileDescriptor = dup(self.tunnelFileDescriptor)
        if duplicateFileDescriptor == -1 {
            return .failure(.cannotDuplicateSocketDescriptor(errno))
        }

        let handle = resolvedConfiguration
            .uapiConfiguration()
            .toRawWireguardConfigString()
            .withCString { wgTurnOn($0, duplicateFileDescriptor) }

        if handle >= 0 {
            self.wireguardHandle = handle

            return .success(())
        } else {
            // `wgTurnOn` does not cover all of the code paths and may leave the file descriptor
            // open on failure
            if close(duplicateFileDescriptor) == -1 {
                self.logger.warning("Failed to close the duplicate tunnel file descriptor. Error: \(errno)")
            }

            return .failure(.start(handle))
        }
    }

    private func stopWireguardBackend() {
        guard let handle = self.wireguardHandle else { return }

        wgTurnOff(handle)
        self.wireguardHandle = nil
    }

    private class func setWireguardConfig(handle: Int32, commands: [WireguardCommand]) {
        // Ignore empty payloads
        guard !commands.isEmpty else { return }

        _ = commands.toRawWireguardConfigString()
            .withCString { wgSetConfig(handle, $0) }
    }

    private func resolveConfiguration(_ configuration: WireguardConfiguration)
        -> WireguardConfiguration
    {
        return WireguardConfiguration(
            privateKey: configuration.privateKey,
            peers: resolvePeers(configuration.peers),
            allowedIPs: configuration.allowedIPs
        )
    }

    private func resolvePeers(_ peers: [WireguardPeer]) -> [WireguardPeer] {
        var newPeers = [WireguardPeer]()

        for peer in peers {
            switch resolvePeer(peer) {
            case .success(let resolvedPeer):
                newPeers.append(resolvedPeer)
            case .failure(_):
                // Fix me: Ignore resolution error and carry on with the last known peer
                newPeers.append(peer)
            }
        }

        return newPeers
    }

    private func resolvePeer(_ peer: WireguardPeer) -> Result<WireguardPeer, Error> {
        switch peer.withReresolvedEndpoint() {
        case .success(let resolvedPeer):
            if "\(peer.endpoint.ip)" == "\(resolvedPeer.endpoint.ip)" {
                logger.debug("DNS64: mapped \(resolvedPeer.endpoint.ip) to itself")
            } else {
                logger.debug("DNS64: mapped \(peer.endpoint.ip) to \(resolvedPeer.endpoint.ip)")
            }

            return .success(resolvedPeer)

        case .failure(let error):
            logger.error("Failed to re-resolve the peer: \(peer.endpoint.ip). Error: \(error.localizedDescription)")

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
        networkMonitor.start(queue: workQueue)
        self.networkMonitor = networkMonitor
    }

    private func didReceiveNetworkPathUpdate(path: Network.NWPath) {
        guard self.isStarted else { return }

        self.logger.info("Network change detected. Status: \(path.status), interfaces \(path.availableInterfaces).")

        let oldPathSatisfied = self.isPathSatisfied
        let newPathSatisfied = path.status.isSatisfiable

        self.isPathSatisfied = newPathSatisfied

        switch (oldPathSatisfied, newPathSatisfied)  {
        case (true, false):
            self.logger.info("Stop wireguard backend")
            self.stopWireguardBackend()

        case (false, true), (true, true):
            guard let currentConfiguration = self.configuration else { return }

            self.logger.info("Re-resolve endpoints")

            let resolvedConfiguration = self.resolveConfiguration(currentConfiguration)

            if let handle = self.wireguardHandle {
                let commands = resolvedConfiguration.endpointUapiConfiguration()
                Self.setWireguardConfig(handle: handle, commands: commands)

                wgBumpSockets(handle)
            } else {
                self.logger.info("Start wireguard backend")

                if case .failure(let error) = self.startWireguardBackend(resolvedConfiguration: resolvedConfiguration) {
                    self.logger.error(chainedError: error, message: "Failed to turn on WireGuard")
                }
            }

        case (false, false):
            // No-op: device remains offline
            break
        }
    }
}

/// A enum describing Wireguard log levels defined in `api-ios.go` from `wireguard-apple` repository
enum WireguardLogLevel: Int32 {
    case debug = 0
    case info = 1
    case error = 2

    var loggerLevel: Logger.Level {
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

private extension Network.NWPath.Status {
    /// Returns `true` if the path is potentially satisfiable
    var isSatisfiable: Bool {
        switch self {
        case .requiresConnection, .satisfied:
            return true
        case .unsatisfied:
            return false
        @unknown default:
            return true
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
