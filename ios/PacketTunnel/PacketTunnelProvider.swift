//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import Foundation
import Network
import NetworkExtension
import os

enum PacketTunnelProviderError: Error {
    /// Failure to read the relay cache
    case readRelayCache(RelayCacheError)

    /// Failure to satisfy the relay constraint
    case noRelaySatisfyingConstraint

    /// Missing the persistent keychain reference to the tunnel configuration
    case missingKeychainConfigurationReference

    /// Failure to read the tunnel configuration from Keychain
    case cannotReadTunnelConfiguration(TunnelConfigurationManagerError)

    /// Failure to set network settings
    case setNetworkSettings(Error)

    /// Failure to discover the tunnel device (utun)
    case tunnelDeviceNotFound

    /// Failure to start the Wireguard backend
    case startWireGuardBackend

    /// IPC handler failure
    case ipcHandler(PacketTunnelIpcHandlerError)

    var localizedDescription: String {
        switch self {
        case .readRelayCache(let relayError):
            return "Failure to read the relay cache: \(relayError.localizedDescription)"

        case .noRelaySatisfyingConstraint:
            return "No relay satisfying the given constraint"

        case .missingKeychainConfigurationReference:
            return "Invalid protocol configuration"

        case .cannotReadTunnelConfiguration(let readError):
            return "Cannot read tunnel configuration: \(readError.localizedDescription)"

        case .setNetworkSettings(let systemError):
            return "Failed to set network settings: \(systemError.localizedDescription)"

        case .tunnelDeviceNotFound:
            return "Failed to find the tunnel device descriptor"

        case .startWireGuardBackend:
            return "Failure to start the wireguard backend"

        case .ipcHandler(let ipcError):
            return "Failure to handle the IPC request: \(ipcError.localizedDescription)"
        }
    }
}

/// A wireguard events log
let wireguardLog = OSLog(subsystem: "net.mullvad.vpn.packet-tunnel", category: "Wireguard")

/// A general tunnel provider log
let tunnelProviderLog = OSLog(subsystem: "net.mullvad.vpn.packet-tunnel", category: "Tunnel Provider")

class PacketTunnelProvider: NEPacketTunnelProvider {

    private var handle: Int32?
    private var networkMonitor: NWPathMonitor?
    private var tunnelInterfaceName: String?
    private var packetTunnelSettingsGenerator: PacketTunnelSettingsGenerator?
    private var connectionInfo: TunnelConnectionInfo?
    private let cancellableSet = CancellableSet()

    private var startStopTunnelSubscriber: AnyCancellable?
    private var startedTunnel = false

    private let exclusivityQueue = DispatchQueue(label: "net.mullvad.vpn.packet-tunnel.exclusivity-queue")
    private let executionQueue = DispatchQueue(label: "net.mullvad.vpn.packet-tunnel.execution-queue")
    private let networkMonitorQueue = DispatchQueue(label: "net.mullvad.vpn.packet-tunnel.network-monitor")

    override init() {
        super.init()

        self.configureLogger()
    }

    deinit {
        networkMonitor?.cancel()
    }

    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        os_log(.default, log: tunnelProviderLog, "Start tunnel received.")

        startStopTunnelSubscriber = self.startTunnel()
            .sink(receiveCompletion: { (completion) in
                    switch completion {
                    case .finished:
                        os_log(.default, log: tunnelProviderLog, "Started the tunnel")

                        completionHandler(nil)

                    case .failure(let error):
                        os_log(.error, log: tunnelProviderLog, "Failed to start the tunnel: %{public}s", error.localizedDescription)

                        completionHandler(error)
                    }
            })
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        os_log(.default, log: tunnelProviderLog, "Stop tunnel received.")

        startStopTunnelSubscriber = stopTunnel().sink(receiveCompletion: { (completion) in
            os_log(.default, log: tunnelProviderLog, "Stopped the tunnel")

            completionHandler()
        })
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        PacketTunnelIpcHandler.decodeRequest(messageData: messageData)
            .mapError { PacketTunnelProviderError.ipcHandler($0) }
            .receive(on: executionQueue)
            .flatMap { (request) -> AnyPublisher<AnyEncodable, PacketTunnelProviderError> in
                os_log(.default, log: tunnelProviderLog,
                       "Received IPC request: %{public}s", "\(request)")

                switch request {

                case .reloadConfiguration:
                    return self.reloadTunnel()
                        .map { AnyEncodable(true) }
                        .eraseToAnyPublisher()

                case .tunnelInformation:
                    return Result.Publisher(AnyEncodable(self.connectionInfo))
                        .eraseToAnyPublisher()

                }
        }.flatMap({ (response) in
            return PacketTunnelIpcHandler.encodeResponse(response: response)
                .mapError { PacketTunnelProviderError.ipcHandler($0) }
        }).autoDisposableSink(cancellableSet: cancellableSet, receiveCompletion: { (completion) in
            if case .failure(let error) = completion {
                os_log(.error, log: tunnelProviderLog, "Failed to handle the app message: %{public}s", error.localizedDescription)
                completionHandler?(nil)
            }
        }, receiveValue: { (responseData) in
            completionHandler?(responseData)
        })
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        // Add code here to get ready to sleep.
        completionHandler()
    }

    override func wake() {
        // Add code here to wake up.
    }

    private func configureLogger() {
        wgSetLogger { (level, messagePtr) in
            guard let message = messagePtr.map({ String(cString: $0) }) else { return }

            let logType = WireguardLogLevel(rawValue: level)?.asOSLogType ?? .default

            os_log(logType, log: wireguardLog, "%{public}s", message)
        }
    }

    private func startTunnel() -> AnyPublisher<(), PacketTunnelProviderError> {
        MutuallyExclusive(
            exclusivityQueue: exclusivityQueue,
            executionQueue: executionQueue
        ) { () -> AnyPublisher<(), PacketTunnelProviderError> in
            os_log(.default, log: tunnelProviderLog, "Starting the tunnel")

            self.startedTunnel = true

            return self.setupTunnelNetworkSettings()
                .flatMap { self.startWireguard().publisher }
                .eraseToAnyPublisher()
        }.eraseToAnyPublisher()
    }

    private func stopTunnel() -> AnyPublisher<(), Never> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) { () -> Just<()> in
            os_log(.default, log: tunnelProviderLog, "Stopping the tunnel")

            self.startedTunnel = false

            self.networkMonitor?.cancel()
            self.networkMonitor = nil

            if let handle = self.handle {
                wgTurnOff(handle)
            }

            self.handle = nil

            return Just(())
        }.eraseToAnyPublisher()
    }

    private func reloadTunnel() -> AnyPublisher<(), PacketTunnelProviderError> {
        MutuallyExclusive(exclusivityQueue: exclusivityQueue, executionQueue: executionQueue) {
            () -> AnyPublisher<(), PacketTunnelProviderError> in
            guard self.startedTunnel else {
                os_log(.default, log: tunnelProviderLog,
                       "Ignore reloading tunnel settings. The tunnel has not started yet.")

                return Result.Publisher(()).eraseToAnyPublisher()
            }

            os_log(.default, log: tunnelProviderLog, "Reload tunnel settings")
            return self.setupTunnelNetworkSettings()
                .handleEvents(receiveSubscription: { _ in
                    // Tell the system that the tunnel is about to reconnect with the new endpoint
                    self.reasserting = true
                }, receiveCompletion: { (completion) in
                    switch completion {
                    case .finished:
                        guard let handle = self.handle else { return }

                        os_log(.default, log: tunnelProviderLog, "Replace Wireguard endpoints")

                        _ = self.packetTunnelSettingsGenerator?
                            .wireguardConfigurationForChangingRelays()
                            .withGoString { wgSetConfig(handle, $0) }

                    case .failure(let error):
                        os_log(.default, log: tunnelProviderLog,
                               "Failed to set the new tunnel settings: %{public}s",
                               error.localizedDescription)
                    }

                    // Tell the system that the tunnel has finished reconnecting
                    self.reasserting = false
                }, receiveCancel: {
                    // Tell the system that the tunnel has finished reconnecting
                    // in the event of task cancellation
                    self.reasserting = false
                }).eraseToAnyPublisher()
        }.eraseToAnyPublisher()
    }

    private func setupTunnelNetworkSettings() -> AnyPublisher<(), PacketTunnelProviderError> {
        return readTunnelConfigurationFromKeychain().publisher
            .flatMap { (tunnelConfiguration) in
                return self.selectRelayEndpoint(tunnelConfiguration: tunnelConfiguration)
                    .receive(on: self.executionQueue)
                    .map({ (result) -> MullvadEndpoint in
                        os_log(.default, log: tunnelProviderLog, "Selected relay: %{public}s", result.relay.hostname)

                        self.connectionInfo = TunnelConnectionInfo(
                            ipv4Relay: result.endpoint.ipv4Relay,
                            ipv6Relay: result.endpoint.ipv6Relay,
                            hostname: result.relay.hostname)

                        return result.endpoint
                    })
                    .flatMap({ (endpoint) -> AnyPublisher<(), PacketTunnelProviderError> in
                        let settingsGenerator = PacketTunnelSettingsGenerator(
                            mullvadEndpoint: endpoint,
                            tunnelConfiguration: tunnelConfiguration)

                        let networkSettings = settingsGenerator.networkSettings()

                        self.packetTunnelSettingsGenerator = settingsGenerator

                        os_log(.default, log: tunnelProviderLog, "Set tunnel network settings")

                        return self.setTunnelNetworkSettings(networkSettings)
                            .mapError { (error) in
                                os_log(.error, log: tunnelProviderLog, "Cannot set network settings: %{public}s", error.localizedDescription)

                                return PacketTunnelProviderError.setNetworkSettings(error)
                        }
                        .receive(on: self.executionQueue)
                        .eraseToAnyPublisher()
                    })
        }.eraseToAnyPublisher()
    }

    private func readTunnelConfigurationFromKeychain() -> Result<TunnelConfiguration, PacketTunnelProviderError> {
        guard let keychainReference = (protocolConfiguration as? NETunnelProviderProtocol)?.passwordReference else {
            return .failure(.missingKeychainConfigurationReference)
        }

        return TunnelConfigurationManager.load(persistentKeychainRef: keychainReference)
            .mapError { PacketTunnelProviderError.cannotReadTunnelConfiguration($0) }
    }

    private func selectRelayEndpoint(tunnelConfiguration: TunnelConfiguration) -> AnyPublisher<RelaySelectorResult, PacketTunnelProviderError> {
        return RelaySelector.loadedFromRelayCache()
            .mapError { PacketTunnelProviderError.readRelayCache($0) }
            .flatMap { (relaySelector) -> Result<RelaySelectorResult, PacketTunnelProviderError>.Publisher in
                return relaySelector
                    .evaluate(with: tunnelConfiguration.relayConstraints)
                    .flatMap { .init($0) } ?? .init(.noRelaySatisfyingConstraint)
        }.eraseToAnyPublisher()
    }

    private func startWireguard() -> Result<(), PacketTunnelProviderError> {
        let tunnelSettingsGenerator = self.packetTunnelSettingsGenerator!

        let fileDescriptor = self.getTunnelInterfaceDescriptor()
        if fileDescriptor < 0 {
            os_log(.error, log: tunnelProviderLog, "Cannot find the file descriptor for socket.")
            return .failure(.tunnelDeviceNotFound)
        }

        self.tunnelInterfaceName = self.getInterfaceName(fileDescriptor)

        os_log(.default, log: tunnelProviderLog, "Tunnel interface is %{public}s", self.tunnelInterfaceName ?? "unknown")

        let handle = tunnelSettingsGenerator.entireWireguardConfiguration()
            .withGoString { wgTurnOn($0, fileDescriptor) }

        if handle < 0 {
            os_log(.error, log: tunnelProviderLog, "Failed to start the Wireguard backend, wgTurnOn returned %{public}d", handle)

            return .failure(.startWireGuardBackend)
        }

        self.handle = handle

        startNetworkMonitor()

        return .success(())
    }

    private func startNetworkMonitor() {
        self.networkMonitor?.cancel()

        let networkMonitor = NWPathMonitor()
        networkMonitor.pathUpdateHandler = { [weak self] in self?.didReceiveNetworkPathUpdate(path: $0) }
        networkMonitor.start(queue: networkMonitorQueue)
        self.networkMonitor = networkMonitor
    }

    private func didReceiveNetworkPathUpdate(path: Network.NWPath) {
        executionQueue.async {
            guard let handle = self.handle else { return }

            os_log(.default, log: tunnelProviderLog,
                   "Network change detected with %{public}s route and interface order: %{public}s",
                   "\(path.status)",
                "\(path.availableInterfaces.map { $0.name }.joined(separator: ", "))"
            )

            _ = self.packetTunnelSettingsGenerator?
                .wireguardConfigurationWithReresolvedEndpoints()
                .withGoString { wgSetConfig(handle, $0) }

            wgBumpSockets(handle)
        }
    }
}


extension PacketTunnelProvider {

    fileprivate func getTunnelInterfaceDescriptor() -> Int32 {
        return packetFlow.value(forKeyPath: "socket.fileDescriptor") as? Int32 ?? -1
    }

    fileprivate func getInterfaceName(_ fileDescriptor: Int32) -> String? {
        var buffer = [UInt8](repeating: 0, count: Int(IFNAMSIZ))

        return buffer.withUnsafeMutableBufferPointer({ (mutableBufferPointer) -> String? in
            guard let baseAddress = mutableBufferPointer.baseAddress else { return nil }

            var ifnameSize = socklen_t(IFNAMSIZ)
            let result = getsockopt(
                fileDescriptor,
                2 /* SYSPROTO_CONTROL */,
                2 /* UTUN_OPT_IFNAME */,
                baseAddress,
                &ifnameSize)

            if result == 0 {
                return String(cString: baseAddress)
            } else {
                return nil
            }
        })
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

extension String {
    func withGoString<R>(_ block: (_ goString: gostring_t) throws -> R) rethrows -> R {
        return try withCString { try block(gostring_t(p: $0, n: utf8.count)) }
    }
}

/// A enum describing the Wireguard log levels defined in api-ios.go from wireguard-apple repository
enum WireguardLogLevel: Int32 {
    case debug = 0
    case info = 1
    case error = 2

    var asOSLogType: OSLogType {
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

extension NETunnelProvider {

    func setTunnelNetworkSettings(_ tunnelNetworkSettings: NETunnelNetworkSettings?) -> Future<(), Error> {
        return Future { (fulfill) in
            self.setTunnelNetworkSettings(tunnelNetworkSettings) { (error) in
                if let error = error {
                    fulfill(.failure(error))
                } else {
                    fulfill(.success(()))
                }
            }
        }
    }

}
