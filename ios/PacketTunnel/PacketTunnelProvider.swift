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

    /// Flag indicating device is revoked or not.
    public var isDeviceRevoked = false

    /// Flag indicating that account expiry should be set again.
    public var accountExpiry: Date?

    /// Flag counting number of failed attempts happened.
    private var numberOfFailedAttempts = 0

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
    private lazy var accountProxy = REST.ProxyFactory.shared.createAccountsProxy()

    /// Device data request proxy
    private lazy var deviceProxy = REST.ProxyFactory.shared.createDevicesProxy()

    /// OperationQueue used for gathering account and device information from network requests and
    /// unifying results in a single scope.
    private let operationQueue = AsyncOperationQueue()

    /// Returns `PacketTunnelStatus` used for sharing with main bundle process.
    private var packetTunnelStatus: PacketTunnelStatus {
        return PacketTunnelStatus(
            lastError: lastError?.localizedDescription,
            isNetworkReachable: isNetworkReachable,
            isDeviceRevoked: isDeviceRevoked,
            accountExpiry: accountExpiry,
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

        super.init()

        REST.TransportRegistry.shared.setTransport(
            URLSessionTransport(urlSession: urlSession)
        )

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
                message: "Failed to start the tunnel."
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

    private enum FailedToEstablishConnectionError: Error {
        case unableToReadCredentials
        case fetchingAccountDataFinishedWith(error: Error)
        case fetchingDeviceDataFinishedWith(error: REST.Error)
        case ranOutOfTime(newExpiry: Date)
    }

    private func createGetAccountDataOperation(accountNumber: String)
        -> ResultOperation<REST.AccountData, REST.Error>
    {
        let operation = ResultBlockOperation<REST.AccountData, REST.Error>(
            dispatchQueue: dispatchQueue
        )

        operation.setExecutionBlock { operation in
            let task = self.accountProxy.getAccountData(
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
            let task = self.deviceProxy.getDevice(
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

    private func verifyData(
        accountNumber: String,
        deviceIdentifier: String,
        completion: @escaping (FailedToEstablishConnectionError?) -> Void
    ) {
        let accountOperation = createGetAccountDataOperation(accountNumber: accountNumber)
        let deviceOperation = createGetDeviceDataOperation(
            accountNumber: accountNumber,
            identifier: deviceIdentifier
        )

        let completionOperation = AsyncBlockOperation(dispatchQueue: dispatchQueue) {
            let accountCompletion = accountOperation.completion
            let deviceCompletion = deviceOperation.completion

            if let accountData = accountCompletion?.value {
                if accountData.expiry > Date() {
                    completion(.ranOutOfTime(newExpiry: accountData.expiry))
                }
            }

            if let deviceCompletionError = deviceCompletion?.error {
                completion(.fetchingDeviceDataFinishedWith(error: deviceCompletionError))
            }
        }

        completionOperation.addDependencies([accountOperation, deviceOperation])

        operationQueue.addOperations(
            [accountOperation, deviceOperation, completionOperation],
            waitUntilFinished: false
        )
    }

    // Introduce a new retry strategy with max amount
    private func notifyGUIWithState(with failure: FailedToEstablishConnectionError) {
        switch failure {
        case .unableToReadCredentials:
            providerLogger.debug("Unable to read credentials from keychain")

        case .fetchingAccountDataFinishedWith:
            // Attemp to Retry the request
            break

        case let .fetchingDeviceDataFinishedWith(error):
            if error.compareErrorCode(.deviceNotFound) {
                isDeviceRevoked = true
            } else {
                // Attemp to Retry the request
            }

        case let .ranOutOfTime(newExpiryDate):
            // Pass new expiry to GUI
            accountExpiry = newExpiryDate
            // Stop monitoring tunnel
            tunnelMonitor.stop()
        }
    }

    private func startDiagnosticConnectionRecoveryFailingReason(
        successCompletionHandler: @escaping () -> Void
    ) {
        providerLogger.debug("Failed to recover connection, started diagnostic process")

        guard let deviceState = try? SettingsManager.readDeviceState(),
              let accountData = deviceState.accountData,
              let deviceIdentifier = deviceState.deviceData?.identifier
        else {
            notifyGUIWithState(with: .unableToReadCredentials)
            return
        }

        verifyData(
            accountNumber: accountData.number,
            deviceIdentifier: deviceIdentifier
        ) { [weak self] result in
            guard let self = self else { return }
            switch result {
            case .none:
                self.numberOfFailedAttempts = 0

            case let .some(failure):
                self.notifyGUIWithState(with: failure)
            }

            self.providerLogger.debug("Recover connection. Picking next relay...")
            self.reconnectTunnel(to: .automatic, completionHandler: successCompletionHandler)
        }
    }

    // MARK: - TunnelMonitorDelegate

    func tunnelMonitorDidDetermineConnectionEstablished(_ tunnelMonitor: TunnelMonitor) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        providerLogger.debug("Connection established.")

        startTunnelCompletionHandler?()
        startTunnelCompletionHandler = nil

        reassertTunnelCompletionHandler?()
        reassertTunnelCompletionHandler = nil

        setReconnecting(false)
    }

    func tunnelMonitorDelegate(
        _ tunnelMonitor: TunnelMonitor,
        shouldHandleConnectionRecoveryWithCompletion completionHandler: @escaping () -> Void
    ) {
        dispatchPrecondition(condition: .onQueue(dispatchQueue))

        guard numberOfFailedAttempts.isMultiple(of: 2) else {
            startDiagnosticConnectionRecoveryFailingReason(
                successCompletionHandler: completionHandler
            )
            return
        }

        numberOfFailedAttempts += 1

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
}

/// Enum describing the next relay to connect to.
private enum NextRelay {
    /// Connect to pre-selected relay.
    case set(RelaySelectorResult)

    /// Determine next relay using relay selector.
    case automatic
}
