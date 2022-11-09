//
//  PacketTunnelProvider.swift
//  PacketTunnel
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Network
import NetworkExtension
import Operations
import RelayCache
import RelaySelector
import TunnelProviderMessaging
import WireGuardKit

class PacketTunnelProvider: NEPacketTunnelProvider, TunnelMonitorDelegate {
    /// Tunnel provider logger.
    private let providerLogger: Logger

    /// WireGuard adapter logger.
    private let tunnelLogger: Logger

    /// Internal queue.
    private let dispatchQueue = DispatchQueue(label: "PacketTunnel", qos: .utility)

    /// WireGuard adapter.
    private var adapter: WireGuardAdapter!

    /// Raised once tunnel establishes connection in the very first time, before calling the system
    /// completion handler passed into `startTunnel`.
    private var isConnected = false

    /// Flag indicating whether network is reachable.
    private var isNetworkReachable = true

    /// Struct holding result of the last device check.
    private var deviceCheck: DeviceCheck?

    /// Number of consecutive connection failure attempts.
    private var numberOfFailedAttempts: UInt = 0

    /// Last runtime error.
    private var lastError: Error?

    /// Relay cache.
    private let relayCache = RelayCache(
        securityGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier
    )!

    /// Current selector result.
    private var selectorResult: RelaySelectorResult?

    /// URL session used for proxy requests.
    private let urlSession = REST.makeURLSession()

    /// List of all proxied network requests bypassing VPN.
    private var proxiedRequests: [UUID: URLSessionDataTask] = [:]

    /// A system completion handler passed from startTunnel and saved for later use once the
    /// connection is established.
    private var startTunnelCompletionHandler: (() -> Void)?

    /// A completion handler passed during reassertion and saved for later use once the connection
    /// is reestablished.
    private var reassertTunnelCompletionHandler: (() -> Void)?

    /// Tunnel monitor.
    private var tunnelMonitor: TunnelMonitor!

    /// Account data request proxy
    private let accountsProxy: REST.AccountsProxy

    /// Device data request proxy
    private let devicesProxy: REST.DevicesProxy

    /// Last device check task.
    private var checkDeviceStateTask: Cancellable?

    /// Internal operation queue.
    private let operationQueue = AsyncOperationQueue()

    /// Returns `PacketTunnelStatus` used for sharing with main bundle process.
    private var packetTunnelStatus: PacketTunnelStatus {
        return PacketTunnelStatus(
            lastError: lastError?.localizedDescription,
            isNetworkReachable: isNetworkReachable,
            deviceCheck: deviceCheck,
            tunnelRelay: selectorResult?.packetTunnelRelay
        )
    }

    override init() {
        let pid = ProcessInfo.processInfo.processIdentifier

        var metadata = Logger.Metadata()
        metadata["pid"] = .string("\(pid)")

        initLoggingSystem(
            bundleIdentifier: Bundle.main.bundleIdentifier!,
            applicationGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier,
            metadata: metadata
        )

        providerLogger = Logger(label: "PacketTunnelProvider")
        tunnelLogger = Logger(label: "WireGuard")

        let addressCache = REST.AddressCache(
            securityGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier,
            isReadOnly: true
        )!
        let urlSessionTransport = REST.URLSessionTransport(urlSession: urlSession)
        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: { () -> RESTTransport? in
                return urlSessionTransport
            },
            addressCache: addressCache
        )
        accountsProxy = proxyFactory.createAccountsProxy()
        devicesProxy = proxyFactory.createDevicesProxy()

        super.init()

        adapter = WireGuardAdapter(
            with: self,
            shouldHandleReasserting: false,
            logHandler: { [weak self] logLevel, message in
                self?.dispatchQueue.async {
                    self?.tunnelLogger.log(level: logLevel.loggerLevel, "\(message)")
                }
            }
        )

        tunnelMonitor = TunnelMonitor(queue: dispatchQueue, adapter: adapter)
        tunnelMonitor.delegate = self
    }

    override func startTunnel(
        options: [String: NSObject]?,
        completionHandler: @escaping (Error?) -> Void
    ) {
        let tunnelOptions = PacketTunnelOptions(rawOptions: options ?? [:])
        var appSelectorResult: RelaySelectorResult?

        // Parse relay selector from tunnel options.
        do {
            appSelectorResult = try tunnelOptions.getSelectorResult()

            switch appSelectorResult {
            case let .some(selectorResult):
                providerLogger.debug(
                    "Start the tunnel via app, connect to \(selectorResult.relay.hostname)."
                )

            case .none:
                if tunnelOptions.isOnDemand() {
                    providerLogger.debug("Start the tunnel via on-demand rule.")
                } else {
                    providerLogger.debug("Start the tunnel via system.")
                }
            }
        } catch {
            providerLogger.debug("Start the tunnel via app.")
            providerLogger.error(
                error: error,
                message: """
                Failed to decode relay selector result passed from the app. \
                Will continue by picking new relay.
                """
            )
        }

        // Read tunnel configuration.
        let tunnelConfiguration: PacketTunnelConfiguration
        do {
            let initialRelay: NextRelay = appSelectorResult.map { .set($0) } ?? .automatic

            tunnelConfiguration = try makeConfiguration(initialRelay)
        } catch {
            providerLogger.error(
                error: error,
                message: "Failed to read tunnel configuration when starting the tunnel."
            )

            completionHandler(error)
            return
        }

        // Set tunnel status.
        dispatchQueue.async {
            let selectorResult = tunnelConfiguration.selectorResult
            self.selectorResult = selectorResult
            self.providerLogger.debug("Set tunnel relay to \(selectorResult.relay.hostname).")
        }

        // Start tunnel.
        adapter.start(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
            self.dispatchQueue.async {
                if let error = error {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to start the tunnel."
                    )

                    completionHandler(error)
                } else {
                    self.providerLogger.debug("Started the tunnel.")

                    self.startTunnelCompletionHandler = { [weak self] in
                        self?.isConnected = true
                        completionHandler(nil)
                    }

                    self.tunnelMonitor.start(
                        probeAddress: tunnelConfiguration.selectorResult.endpoint.ipv4Gateway
                    )
                }
            }
        }
    }

    override func stopTunnel(
        with reason: NEProviderStopReason,
        completionHandler: @escaping () -> Void
    ) {
        providerLogger.debug("Stop the tunnel: \(reason)")

        dispatchQueue.async {
            self.tunnelMonitor.stop()
            self.checkDeviceStateTask?.cancel()
            self.checkDeviceStateTask = nil
            self.startTunnelCompletionHandler = nil
            self.reassertTunnelCompletionHandler = nil
        }

        adapter.stop { error in
            self.dispatchQueue.async {
                if let error = error {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to stop the tunnel gracefully."
                    )
                } else {
                    self.providerLogger.debug("Stopped the tunnel.")
                }
                completionHandler()
            }
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        dispatchQueue.async {
            let message: TunnelProviderMessage
            do {
                message = try TunnelProviderMessage(messageData: messageData)
            } catch {
                self.providerLogger.error(
                    error: error,
                    message: "Failed to decode the app message."
                )

                completionHandler?(nil)
                return
            }

            self.providerLogger.debug("Received app message: \(message)")

            switch message {
            case let .reconnectTunnel(appSelectorResult):
                self.providerLogger.debug("Reconnecting the tunnel...")

                let nextRelay: NextRelay = (appSelectorResult ?? self.selectorResult)
                    .map { .set($0) } ?? .automatic

                self.reconnectTunnel(to: nextRelay)

                completionHandler?(nil)

            case .getTunnelStatus:
                var response: Data?
                do {
                    response = try TunnelProviderReply(self.packetTunnelStatus).encode()
                } catch {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to encode tunnel status reply."
                    )
                }

                completionHandler?(response)

            case let .sendURLRequest(proxyRequest):
                let task = self.urlSession
                    .dataTask(with: proxyRequest.urlRequest) { [weak self] data, response, error in
                        guard let self = self else { return }

                        self.dispatchQueue.async {
                            self.proxiedRequests.removeValue(forKey: proxyRequest.id)

                            var reply: Data?
                            do {
                                let response = ProxyURLResponse(
                                    data: data,
                                    response: response,
                                    error: error
                                )
                                reply = try TunnelProviderReply(response).encode()
                            } catch {
                                self.providerLogger.error(
                                    error: error,
                                    message: "Failed to encode ProxyURLResponse."
                                )
                            }

                            completionHandler?(reply)
                        }
                    }

                self.proxiedRequests[proxyRequest.id] = task

                task.resume()

            case let .cancelURLRequest(id):
                let task = self.proxiedRequests.removeValue(forKey: id)

                task?.cancel()
            }
        }
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        tunnelMonitor.onSleep {
            completionHandler()
        }
    }

    override func wake() {
        tunnelMonitor.onWake()
    }

    // MARK: - TunnelMonitorDelegate

    func tunnelMonitorDidDetermineConnectionEstablished(_ tunnelMonitor: TunnelMonitor) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Connection established.")

        startTunnelCompletionHandler?()
        startTunnelCompletionHandler = nil

        reassertTunnelCompletionHandler?()
        reassertTunnelCompletionHandler = nil

        numberOfFailedAttempts = 0

        checkDeviceStateTask?.cancel()
        checkDeviceStateTask = nil

        deviceCheck = nil

        setReconnecting(false)
    }

    func tunnelMonitorDelegate(
        _ tunnelMonitor: TunnelMonitor,
        shouldHandleConnectionRecoveryWithCompletion completionHandler: @escaping () -> Void
    ) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        let (value, isOverflow) = numberOfFailedAttempts.addingReportingOverflow(1)
        numberOfFailedAttempts = isOverflow ? 0 : value

        if numberOfFailedAttempts.isMultiple(of: 2) {
            startDeviceCheck()
        }

        providerLogger.debug("Recover connection. Picking next relay...")

        reconnectTunnel(to: .automatic, completionHandler: completionHandler)
    }

    func tunnelMonitor(
        _ tunnelMonitor: TunnelMonitor,
        networkReachabilityStatusDidChange isNetworkReachable: Bool
    ) {
        guard self.isNetworkReachable != isNetworkReachable else { return }

        self.isNetworkReachable = isNetworkReachable

        // Switch tunnel into reconnecting state when offline.
        if !isNetworkReachable {
            setReconnecting(true)
        }
    }

    // MARK: - Private

    private func setReconnecting(_ reconnecting: Bool) {
        // Raise reasserting flag, but only if tunnel has already moved to connected state once.
        // Otherwise keep the app in connecting state until it manages to establish the very first
        // connection.
        if isConnected {
            reasserting = reconnecting
        }
    }

    private func makeConfiguration(_ nextRelay: NextRelay)
        throws -> PacketTunnelConfiguration
    {
        let deviceState = try SettingsManager.readDeviceState()
        let tunnelSettings = try SettingsManager.readSettings()
        let selectorResult: RelaySelectorResult

        switch nextRelay {
        case .automatic:
            selectorResult = try selectRelayEndpoint(
                relayConstraints: tunnelSettings.relayConstraints
            )
        case let .set(aSelectorResult):
            selectorResult = aSelectorResult
        }

        return PacketTunnelConfiguration(
            deviceState: deviceState,
            tunnelSettings: tunnelSettings,
            selectorResult: selectorResult
        )
    }

    private func reconnectTunnel(to nextRelay: NextRelay, completionHandler: (() -> Void)? = nil) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        // Read tunnel configuration.
        let tunnelConfiguration: PacketTunnelConfiguration
        do {
            tunnelConfiguration = try makeConfiguration(nextRelay)
        } catch {
            providerLogger.error(
                error: error,
                message: "Failed produce new configuration."
            )
            completionHandler?()
            return
        }

        // Copy old relay.
        let oldSelectorResult = selectorResult
        let newTunnelRelay = tunnelConfiguration.selectorResult.packetTunnelRelay

        // Update tunnel status.
        selectorResult = tunnelConfiguration.selectorResult

        providerLogger.debug("Set tunnel relay to \(newTunnelRelay.hostname).")
        setReconnecting(true)

        adapter.update(tunnelConfiguration: tunnelConfiguration.wgTunnelConfig) { error in
            self.dispatchQueue.async {
                self.lastError = error

                if let error = error {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to update WireGuard configuration."
                    )

                    // Revert to previously used relay selector as it's very likely that we keep
                    // using previous configuration.
                    self.selectorResult = oldSelectorResult
                    self.providerLogger.debug(
                        "Reset tunnel relay to \(oldSelectorResult?.relay.hostname ?? "none")."
                    )
                    self.reassertTunnelCompletionHandler = nil
                    self.setReconnecting(false)

                    completionHandler?()
                } else {
                    self.reassertTunnelCompletionHandler = completionHandler

                    self.tunnelMonitor.start(
                        probeAddress: tunnelConfiguration.selectorResult.endpoint.ipv4Gateway
                    )
                }
            }
        }
    }

    /// Load relay cache with potential networking to refresh the cache and pick the relay for the
    /// given relay constraints.
    private func selectRelayEndpoint(relayConstraints: RelayConstraints) throws
        -> RelaySelectorResult
    {
        let cachedRelayList = try relayCache.read()

        return try RelaySelector.evaluate(
            relays: cachedRelayList.relays,
            constraints: relayConstraints
        )
    }

    // MARK: - Device check

    /// Fetch account and device data to verify account expiry and device status.
    /// Saves the result into deviceCheck
    private func startDeviceCheck() {
        let deviceState: DeviceState
        do {
            deviceState = try SettingsManager.readDeviceState()
        } catch {
            providerLogger.error(
                error: error,
                message: "Failed to read device states"
            )
            return
        }

        guard case let .loggedIn(storedAccountData, storedDeviceData) = deviceState else {
            return
        }

        providerLogger.debug("Start device check")

        let accountOperation = createGetAccountDataOperation(
            accountNumber: storedAccountData.number
        )
        let deviceOperation = createGetDeviceDataOperation(
            accountNumber: storedAccountData.number,
            identifier: storedDeviceData.identifier
        )

        let completionOperation = AsyncBlockOperation(dispatchQueue: dispatchQueue) { [weak self] in
            guard let self = self else { return }

            var newAccountExpiry: Date?
            var newDeviceRevoked: Bool?

            switch accountOperation.completion {
            case let .failure(error):
                self.providerLogger.error(
                    error: error,
                    message: "Failed to fetch account data."
                )

            case let .success(accountData):
                newAccountExpiry = accountData.expiry

            case .none, .cancelled: break
            }

            switch deviceOperation.completion {
            case let .failure(error):
                if error.compareErrorCode(.deviceNotFound) {
                    newDeviceRevoked = true
                } else {
                    self.providerLogger.error(
                        error: error,
                        message: "Failed to fetch device data."
                    )
                }

            case .none, .cancelled, .success: break
            }

            if var deviceCheck = self.deviceCheck {
                deviceCheck.update(
                    accountExpiry: newAccountExpiry,
                    isDeviceRevoked: newDeviceRevoked
                )

                self.deviceCheck = deviceCheck
            } else {
                self.deviceCheck = DeviceCheck(
                    identifier: UUID(),
                    isDeviceRevoked: newDeviceRevoked,
                    accountExpiry: newAccountExpiry
                )
            }

            if newDeviceRevoked ?? false {
                self.tunnelMonitor.stop()
            }
        }

        completionOperation.addDependencies([accountOperation, deviceOperation])

        let groupOperation = GroupOperation(operations: [
            accountOperation,
            deviceOperation,
            completionOperation,
        ])
        checkDeviceStateTask = groupOperation

        operationQueue.addOperation(groupOperation)
    }

    private func createGetAccountDataOperation(accountNumber: String)
        -> ResultOperation<REST.AccountData, REST.Error>
    {
        let operation = ResultBlockOperation<REST.AccountData, REST.Error>(
            dispatchQueue: dispatchQueue
        )

        operation.setExecutionBlock { operation in
            let task = self.accountsProxy.getAccountData(
                accountNumber: accountNumber,
                retryStrategy: .noRetry
            ) { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        return operation
    }

    private func createGetDeviceDataOperation(accountNumber: String, identifier: String)
        -> ResultOperation<REST.Device, REST.Error>
    {
        let operation = ResultBlockOperation<REST.Device, REST.Error>(dispatchQueue: dispatchQueue)

        operation.setExecutionBlock { operation in
            let task = self.devicesProxy.getDevice(
                accountNumber: accountNumber,
                identifier: identifier,
                retryStrategy: .noRetry
            ) { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                task.cancel()
            }
        }

        return operation
    }
}

/// Enum describing the next relay to connect to.
private enum NextRelay {
    /// Connect to pre-selected relay.
    case set(RelaySelectorResult)

    /// Determine next relay using relay selector.
    case automatic
}

extension DeviceCheck {
    mutating func update(accountExpiry: Date?, isDeviceRevoked: Bool?) {
        var shouldChangeIdentifier = false

        if let accountExpiry = accountExpiry, self.accountExpiry != accountExpiry {
            shouldChangeIdentifier = true
            self.accountExpiry = accountExpiry
        }

        if let isDeviceRevoked = isDeviceRevoked, self.isDeviceRevoked != isDeviceRevoked {
            shouldChangeIdentifier = true
            self.isDeviceRevoked = isDeviceRevoked
        }

        if shouldChangeIdentifier {
            identifier = UUID()
        }
    }
}
