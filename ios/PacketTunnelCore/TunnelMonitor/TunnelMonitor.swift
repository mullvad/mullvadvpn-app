//
//  TunnelMonitor.swift
//  PacketTunnelCore
//
//  Created by pronebird on 09/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import protocol Network.IPAddress
import struct Network.IPv4Address

/// Interval for periodic heartbeat ping issued when traffic is flowing.
/// Should help to detect connectivity issues on networks that drop traffic in one of directions,
/// regardless if tx/rx counters are being updated.
private let heartbeatPingInterval: TimeInterval = 10

/// Heartbeat timeout that once exceeded triggers next heartbeat to be sent.
private let heartbeatReplyTimeout: TimeInterval = 3

/// Timeout used to determine if there was a network activity lately.
private let trafficFlowTimeout: TimeInterval = heartbeatPingInterval * 0.5

/// Ping timeout.
private let pingTimeout: TimeInterval = 15

/// Interval to wait before sending next ping.
private let pingDelay: TimeInterval = 3

/// Initial timeout when establishing connection.
private let initialEstablishTimeout: TimeInterval = 4

/// Multiplier applied to `establishTimeout` on each failed connection attempt.
private let establishTimeoutMultiplier: UInt32 = 2

/// Maximum timeout when establishing connection.
private let maxEstablishTimeout: TimeInterval = pingTimeout

/// Connectivity check periodicity.
private let connectivityCheckInterval: TimeInterval = 1

/// Inbound traffic timeout used when outbound traffic was registered prior to inbound traffic.
private let inboundTrafficTimeout: TimeInterval = 5

/// Traffic timeout applied when both tx/rx counters remain stale, i.e no traffic flowing.
/// Ping is issued after that timeout is exceeded.s
private let trafficTimeout: TimeInterval = 120

public final class TunnelMonitor: TunnelMonitorProtocol {
    /// Connection state.
    private enum ConnectionState {
        /// Initialized and doing nothing.
        case stopped

        /// Preparing to start.
        /// Intermediate state before receiving the first path update.
        case pendingStart

        /// Establishing connection.
        case connecting

        /// Connection is established.
        case connected

        /// Delegate is recovering connection.
        /// Delegate has to call `start(probeAddress:)` to complete recovery and resume monitoring.
        case recovering

        /// Waiting for network connectivity.
        case waitingConnectivity
    }

    /// Tunnel monitor state.
    private struct State {
        /// Current connection state.
        var connectionState: ConnectionState = .stopped

        /// Network counters.
        var netStats = WgStats()

        /// Ping stats.
        var pingStats = PingStats()

        /// Reference date used to determine if timeout has occurred.
        var timeoutReference = Date()

        /// Last seen change in rx counter.
        var lastSeenRx: Date?

        /// Last seen change in tx counter.
        var lastSeenTx: Date?

        /// Whether periodic heartbeat is suspended.
        var isHeartbeatSuspended = false

        /// Retry attempt.
        var retryAttempt: UInt32 = 0

        func evaluateConnection(now: Date, pingTimeout: TimeInterval) -> ConnectionEvaluation {
            switch connectionState {
            case .connecting:
                if now.timeIntervalSince(timeoutReference) >= pingTimeout {
                    return .pingTimeout
                }

                guard let lastRequestDate = pingStats.lastRequestDate else {
                    return .sendInitialPing
                }

                if now.timeIntervalSince(lastRequestDate) >= pingDelay {
                    return .sendNextPing
                }

            case .connected:
                if now.timeIntervalSince(timeoutReference) >= pingTimeout, !isHeartbeatSuspended {
                    return .pingTimeout
                }

                guard let lastRequestDate = pingStats.lastRequestDate else {
                    return .sendInitialPing
                }

                let timeSinceLastPing = now.timeIntervalSince(lastRequestDate)
                if let lastReplyDate = pingStats.lastReplyDate,
                   lastRequestDate.timeIntervalSince(lastReplyDate) >= heartbeatReplyTimeout,
                   timeSinceLastPing >= pingDelay, !isHeartbeatSuspended {
                    return .retryHeartbeatPing
                }

                guard let lastSeenRx, let lastSeenTx else { return .ok }

                let rxTimeElapsed = now.timeIntervalSince(lastSeenRx)
                let txTimeElapsed = now.timeIntervalSince(lastSeenTx)

                if timeSinceLastPing >= heartbeatPingInterval {
                    // Send heartbeat if traffic is flowing.
                    if rxTimeElapsed <= trafficFlowTimeout || txTimeElapsed <= trafficFlowTimeout {
                        return .sendHeartbeatPing
                    }

                    if !isHeartbeatSuspended {
                        return .suspendHeartbeat
                    }
                }

                if timeSinceLastPing >= pingDelay {
                    if txTimeElapsed >= trafficTimeout || rxTimeElapsed >= trafficTimeout {
                        return .trafficTimeout
                    }

                    if lastSeenTx > lastSeenRx, rxTimeElapsed >= inboundTrafficTimeout {
                        return .inboundTrafficTimeout
                    }
                }

            default:
                break
            }

            return .ok
        }

        func getPingTimeout() -> TimeInterval {
            switch connectionState {
            case .connecting:
                let multiplier = establishTimeoutMultiplier.saturatingPow(retryAttempt)
                let nextTimeout = initialEstablishTimeout * Double(multiplier)

                if nextTimeout.isFinite, nextTimeout < maxEstablishTimeout {
                    return nextTimeout
                } else {
                    return maxEstablishTimeout
                }

            case .pendingStart, .connected, .waitingConnectivity, .stopped, .recovering:
                return pingTimeout
            }
        }

        mutating func updateNetStats(newStats: WgStats, now: Date) {
            if newStats.bytesReceived > netStats.bytesReceived {
                lastSeenRx = now
            }

            if newStats.bytesSent > netStats.bytesSent {
                lastSeenTx = now
            }

            netStats = newStats
        }

        mutating func updatePingStats(sendResult: PingerSendResult, now: Date) {
            pingStats.requests.updateValue(now, forKey: sendResult.sequenceNumber)
            pingStats.lastRequestDate = now
        }

        mutating func setPingReplyReceived(_ sequenceNumber: UInt16, now: Date) -> Date? {
            guard let pingTimestamp = pingStats.requests.removeValue(forKey: sequenceNumber) else {
                return nil
            }

            pingStats.lastReplyDate = now
            timeoutReference = now

            return pingTimestamp
        }
    }

    /// Ping statistics.
    private struct PingStats {
        /// Dictionary holding sequence and corresponding date when echo request took place.
        var requests = [UInt16: Date]()

        /// Timestamp when last echo request was sent.
        var lastRequestDate: Date?

        /// Timestamp when last echo reply was received.
        var lastReplyDate: Date?
    }

    private let tunnelDeviceInfo: TunnelDeviceInfoProtocol

    private let nslock = NSLock()
    private let timerQueue = DispatchQueue(label: "TunnelMonitor-timerQueue")
    private let eventQueue: DispatchQueue

    private var pinger: PingerProtocol
    private var defaultPathObserver: DefaultPathObserverProtocol
    private var defaultPathObservationToken: DefaultPathObservation?
    private var timer: DispatchSourceTimer?

    private var state = State()
    private var probeAddress: IPv4Address?

    private let logger = Logger(label: "TunnelMonitor")

    private var _onEvent: ((TunnelMonitorEvent) -> Void)?
    public var onEvent: ((TunnelMonitorEvent) -> Void)? {
        set {
            nslock.withLock {
                _onEvent = newValue
            }
        }
        get {
            nslock.withLock {
                return _onEvent
            }
        }
    }

    public init(
        eventQueue: DispatchQueue,
        pinger: PingerProtocol,
        tunnelDeviceInfo: TunnelDeviceInfoProtocol,
        defaultPathObserver: DefaultPathObserverProtocol
    ) {
        self.eventQueue = eventQueue
        self.tunnelDeviceInfo = tunnelDeviceInfo
        self.defaultPathObserver = defaultPathObserver

        self.pinger = pinger
        self.pinger.onReply = { [weak self] reply in
            guard let self else { return }

            switch reply {
            case let .success(sender, sequenceNumber):
                didReceivePing(from: sender, sequenceNumber: sequenceNumber)

            case let .parseError(error):
                logger.error(error: error, message: "Failed to parse ICMP response.")
            }
        }
    }

    deinit {
        stop()
    }

    public func start(probeAddress: IPv4Address) {
        nslock.lock()
        defer { nslock.unlock() }

        if case .stopped = state.connectionState {
            logger.debug("Start with address: \(probeAddress).")
        } else {
            _stop(forRestart: true)
            logger.debug("Restart with address: \(probeAddress).")
        }

        self.probeAddress = probeAddress
        state.connectionState = .pendingStart

        addDefaultPathObserver()
    }

    public func stop() {
        nslock.lock()
        defer { nslock.unlock() }

        _stop()
    }

    public func onWake() {
        nslock.lock()
        defer { nslock.unlock() }

        logger.trace("Wake up.")

        switch state.connectionState {
        case .connecting, .connected:
            startConnectivityCheckTimer()
            addDefaultPathObserver()

        case .waitingConnectivity, .pendingStart:
            addDefaultPathObserver()

        case .stopped, .recovering:
            break
        }
    }

    public func onSleep() {
        nslock.lock()
        defer { nslock.unlock() }

        logger.trace("Prepare to sleep.")

        stopConnectivityCheckTimer()
        removeDefaultPathObserver()
    }

    // MARK: - Private

    private func _stop(forRestart: Bool = false) {
        if case .stopped = state.connectionState {
            return
        }

        if !forRestart {
            logger.debug("Stop tunnel monitor.")
        }

        probeAddress = nil

        removeDefaultPathObserver()
        stopMonitoring(resetRetryAttempt: !forRestart)

        state.connectionState = .stopped
    }

    private func addDefaultPathObserver() {
        defaultPathObservationToken?.invalidate()

        logger.trace("Add default path observer.")

        defaultPathObservationToken = defaultPathObserver.observe { [weak self] nwPath in
            guard let self else { return }

            nslock.withLock {
                self.handleNetworkPathUpdate(nwPath)
            }
        }

        if let currentPath = defaultPathObserver.defaultPath {
            handleNetworkPathUpdate(currentPath)
        }
    }

    private func removeDefaultPathObserver() {
        guard let defaultPathObservationToken else { return }

        logger.trace("Remove default path observer.")

        defaultPathObservationToken.invalidate()
        self.defaultPathObservationToken = nil
    }

    private func checkConnectivity() {
        nslock.lock()
        defer { nslock.unlock() }

        guard let probeAddress, let newStats = getStats(),
              state.connectionState == .connecting || state.connectionState == .connected
        else { return }

        // Check if counters were reset.
        let isStatsReset = newStats.bytesReceived < state.netStats.bytesReceived ||
            newStats.bytesSent < state.netStats.bytesSent

        guard !isStatsReset else {
            logger.trace("Stats was being reset.")
            state.netStats = newStats
            return
        }

        #if DEBUG
        logCounters(currentStats: state.netStats, newStats: newStats)
        #endif

        let now = Date()
        state.updateNetStats(newStats: newStats, now: now)

        let timeout = state.getPingTimeout()
        let evaluation = state.evaluateConnection(now: now, pingTimeout: timeout)

        if evaluation != .ok {
            logger.trace("Evaluation: \(evaluation)")
        }

        switch evaluation {
        case .ok:
            break

        case .pingTimeout:
            startConnectionRecovery()

        case .suspendHeartbeat:
            state.isHeartbeatSuspended = true

        case .sendHeartbeatPing, .retryHeartbeatPing, .sendNextPing, .sendInitialPing,
             .inboundTrafficTimeout, .trafficTimeout:
            if state.isHeartbeatSuspended {
                state.isHeartbeatSuspended = false
                state.timeoutReference = now
            }
            sendPing(to: probeAddress, now: now)
        }
    }

    #if DEBUG
    private func logCounters(currentStats: WgStats, newStats: WgStats) {
        let rxDelta = newStats.bytesReceived.saturatingSubtraction(currentStats.bytesReceived)
        let txDelta = newStats.bytesSent.saturatingSubtraction(currentStats.bytesSent)

        guard rxDelta > 0 || txDelta > 0 else { return }

        logger.trace(
            """
            rx: \(newStats.bytesReceived) (+\(rxDelta)) \
            tx: \(newStats.bytesSent) (+\(txDelta))
            """
        )
    }
    #endif

    private func startConnectionRecovery() {
        removeDefaultPathObserver()
        stopMonitoring(resetRetryAttempt: false)

        state.retryAttempt = state.retryAttempt.saturatingAddition(1)
        state.connectionState = .recovering
        probeAddress = nil

        sendConnectionLostEvent()
    }

    private func sendPing(to receiver: IPv4Address, now: Date) {
        do {
            let sendResult = try pinger.send(to: receiver)
            state.updatePingStats(sendResult: sendResult, now: now)

            logger.trace("Send ping icmp_seq=\(sendResult.sequenceNumber).")
        } catch {
            logger.error(error: error, message: "Failed to send ping.")
        }
    }

    private func handleNetworkPathUpdate(_ networkPath: NetworkPath) {
        let pathStatus = networkPath.status
        let isReachable = pathStatus == .satisfiable || pathStatus == .satisfied

        switch state.connectionState {
        case .pendingStart:
            if isReachable {
                logger.debug("Start monitoring connection.")
                startMonitoring()
                sendNetworkStatusChangeEvent(true)
            } else {
                logger.debug("Wait for network to become reachable before starting monitoring.")
                state.connectionState = .waitingConnectivity
                sendNetworkStatusChangeEvent(false)
            }

        case .waitingConnectivity:
            guard isReachable else { return }

            logger.debug("Network is reachable. Resume monitoring.")
            startMonitoring()
            sendNetworkStatusChangeEvent(true)

        case .connecting, .connected:
            guard !isReachable else { return }

            logger.debug("Network is unreachable. Pause monitoring.")
            state.connectionState = .waitingConnectivity
            stopMonitoring(resetRetryAttempt: true)
            sendNetworkStatusChangeEvent(false)

        case .stopped, .recovering:
            break
        }
    }

    private func didReceivePing(from sender: IPAddress, sequenceNumber: UInt16) {
        nslock.lock()
        defer { nslock.unlock() }

        guard let probeAddress else { return }

        if sender.rawValue != probeAddress.rawValue {
            logger.trace("Got reply from unknown sender: \(sender), expected: \(probeAddress).")
        }

        let now = Date()
        guard let pingTimestamp = state.setPingReplyReceived(sequenceNumber, now: now) else {
            logger.trace("Got unknown ping sequence: \(sequenceNumber).")
            return
        }

        logger.trace({
            let time = now.timeIntervalSince(pingTimestamp) * 1000
            let message = String(
                format: "Received reply icmp_seq=%d, time=%.2f ms.",
                sequenceNumber,
                time
            )
            return Logger.Message(stringLiteral: message)
        }())

        if case .connecting = state.connectionState {
            state.connectionState = .connected
            state.retryAttempt = 0
            sendConnectionEstablishedEvent()
        }
    }

    private func startMonitoring() {
        do {
            guard let interfaceName = tunnelDeviceInfo.interfaceName else {
                logger.debug("Failed to obtain utun interface name.")
                return
            }

            try pinger.openSocket(bindTo: interfaceName)

            state.connectionState = .connecting
            startConnectivityCheckTimer()
        } catch {
            logger.error(error: error, message: "Failed to open socket.")
        }
    }

    private func stopMonitoring(resetRetryAttempt: Bool) {
        stopConnectivityCheckTimer()
        pinger.closeSocket()

        state.netStats = WgStats()
        state.lastSeenRx = nil
        state.lastSeenTx = nil
        state.pingStats = PingStats()

        if resetRetryAttempt {
            state.retryAttempt = 0
        }

        state.isHeartbeatSuspended = false
    }

    private func startConnectivityCheckTimer() {
        let timer = DispatchSource.makeTimerSource(queue: timerQueue)
        timer.setEventHandler { [weak self] in
            self?.checkConnectivity()
        }
        timer.schedule(wallDeadline: .now(), repeating: connectivityCheckInterval)
        timer.activate()

        self.timer?.cancel()
        self.timer = timer

        state.timeoutReference = Date()

        logger.trace("Start connectivity check timer.")
    }

    private func stopConnectivityCheckTimer() {
        guard let timer else { return }

        logger.trace("Stop connectivity check timer.")

        timer.cancel()
        self.timer = nil
    }

    private func sendConnectionEstablishedEvent() {
        eventQueue.async {
            self.onEvent?(.connectionEstablished)
        }
    }

    private func sendConnectionLostEvent() {
        eventQueue.async {
            self.onEvent?(.connectionLost)
        }
    }

    private func sendNetworkStatusChangeEvent(_ isNetworkReachable: Bool) {
        eventQueue.async {
            self.onEvent?(.networkReachabilityChanged(isNetworkReachable))
        }
    }

    private enum ConnectionEvaluation {
        case ok
        case sendInitialPing
        case sendNextPing
        case sendHeartbeatPing
        case retryHeartbeatPing
        case suspendHeartbeat
        case inboundTrafficTimeout
        case trafficTimeout
        case pingTimeout
    }

    private func getStats() -> WgStats? {
        do {
            return try tunnelDeviceInfo.getStats()
        } catch {
            logger.error(error: error, message: "Failed to obtain adapter stats.")

            return nil
        }
    }
}
