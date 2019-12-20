//
//  WireguardDevice.swift
//  PacketTunnel
//
//  Created by pronebird on 16/12/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
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
    enum Error: Swift.Error {
        /// A failure to obtain the tunnel device file descriptor
        case cannotLocateSocketDescriptor

        /// A failure to start the Wireguard backend
        case start(Int32)

        /// A failure that indicates that Wireguard has not been started yet
        case notStarted

        /// A failure that indicates that Wireguard has already been started
        case alreadyStarted

        /// A failure to resolve endpoints
        case resolveEndpoints(Swift.Error)

        var localizedDescription: String {
            switch self {
            case .cannotLocateSocketDescriptor:
                return "Unable to locate the file descriptor for socket."
            case .start(let wgErrorCode):
                return "Failed to start Wireguard. Return code: \(wgErrorCode)"
            case .notStarted:
                return "Wireguard has not been started yet"
            case .alreadyStarted:
                return "Wireguard has already been started"
            case .resolveEndpoints(let resolutionError):
                return "Failed to resolve endpoints: \(resolutionError.localizedDescription)"
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

    /// A subscriber used when resolving peer addresses
    private var peerResolutionSubscriber: AnyCancellable?

    /// A tunnel device descriptor
    private let tunFd: Int32

    /// A wireguard internal handle returned by `wgTurnOn` that's used to associate the calls
    /// with the specific Wireguard tunnel.
    private var wireguardHandle: Int32?

    /// An instance of `WireguardConfiguration`
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

            wgSetLogger { (level, messagePtr) in
                guard let message = messagePtr.map({ String(cString: $0) }) else { return }
                let logType = WireguardLogLevel(rawValue: level) ?? .debug

                WireguardDevice.loggingQueue.async {
                    WireguardDevice.wireguardLogHandler?(logType, message)
                }
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

    func start(configuration: WireguardConfiguration) -> AnyPublisher<(), Error> {
        return Deferred {
            Future { (fulfill) in
                fulfill(self._start(configuration: configuration))
            }
        }.subscribe(on: workQueue)
            .eraseToAnyPublisher()
    }

    func stop() -> AnyPublisher<(), Error> {
        Deferred {
            Future { (fulfill) in
                fulfill(self._stop())
            }
        }.subscribe(on: workQueue)
            .eraseToAnyPublisher()
    }

    func setConfig(configuration: WireguardConfiguration) -> AnyPublisher<(), Error> {
        Deferred {
            Future { (fulfill) in
                fulfill(self._setConfig(configuration: configuration))
            }
        }.subscribe(on: workQueue)
            .eraseToAnyPublisher()
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

    private func _start(configuration: WireguardConfiguration) -> Result<(), Error> {
        guard wireguardHandle == nil else {
            return .failure(.alreadyStarted)
        }

        let handle = configuration.baseline().toRawWireguardConfigString()
            .withGoString { wgTurnOn($0, self.tunFd) }

        if handle < 0 {
            return .failure(.start(handle))
        } else {
            wireguardHandle = handle
            self.configuration = configuration

            startNetworkMonitor()

            return .success(())
        }
    }

    private func _stop() -> Result<(), Error> {
        if let handle = wireguardHandle {
            networkMonitor?.cancel()
            networkMonitor = nil

            wgTurnOff(handle)
            wireguardHandle = nil

            return .success(())
        } else {
            return .failure(.notStarted)
        }
    }

    private func _setConfig(configuration: WireguardConfiguration) -> Result<(), Error> {
        if let handle = wireguardHandle, let activeConfiguration = self.configuration {
            let wireguardCommands = activeConfiguration.transition(to: configuration)

            Self.setWireguardConfig(handle: handle, commands: wireguardCommands)

            self.configuration = configuration

            return .success(())
        } else {
            return .failure(.notStarted)
        }
    }

    private class func setWireguardConfig(handle: Int32, commands: [WireguardCommand]) {
        // Ignore empty payloads
        guard !commands.isEmpty else { return }

        let rawConfig = commands.toRawWireguardConfigString()

        os_log(.info, log: wireguardDeviceLog, "wgSetConfig:\n%{public}s", rawConfig)

        _ = rawConfig.withGoString { wgSetConfig(handle, $0) }
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

            // Re-resolve endpoints on network changes and update Wireguard configuration
            if let activeConfiguration = self.configuration {
                self.peerResolutionSubscriber = activeConfiguration
                    .withReresolvedPeers(maxRetryOnFailure: 1)
                    .mapError { WireguardDevice.Error.resolveEndpoints($0) }
                    .sink(receiveCompletion: { (completion) in
                        switch completion {
                        case .finished:
                            os_log(.debug, log: wireguardDeviceLog, "Re-resolved endpoints")

                        case .failure(let error):
                            os_log(.error, log: wireguardDeviceLog,
                                   "Failed to re-resolve endpoints: %{public}s",
                                   error.localizedDescription)
                        }
                    }, receiveValue: { (reresolvedConfiguration) in
                        let commands = activeConfiguration
                            .transition(to: reresolvedConfiguration)

                        Self.setWireguardConfig(
                            handle: handle,
                            commands: commands
                        )
                    })
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

private extension String {
    func withGoString<R>(_ block: (_ goString: gostring_t) throws -> R) rethrows -> R {
        return try withCString { try block(gostring_t(p: $0, n: utf8.count)) }
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
