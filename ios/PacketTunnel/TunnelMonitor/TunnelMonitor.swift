//
//  TunnelMonitor.swift
//  PacketTunnel
//
//  Created by pronebird on 09/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import NetworkExtension
import WireGuardKit

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

final class TunnelMonitor: PingerDelegate {
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
                   timeSinceLastPing >= pingDelay, !isHeartbeatSuspended
                {
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

        mutating func updatePingStats(sendResult: Pinger.SendResult, now: Date) {
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

    private let adapter: WireGuardAdapter

    private let nslock = NSLock()
    private let eventQueue = DispatchQueue(label: "TunnelMonitor-eventQueue")
    private let delegateQueue: DispatchQueue

    private let pinger: Pinger
    private weak var packetTunnelProvider: NEPacketTunnelProvider?
    private var defaultPathObserver: NSKeyValueObservation?
    private var timer: DispatchSourceTimer?

    private var state = State()
    private var probeAddress: IPv4Address?

    private let logger = Logger(label: "TunnelMonitor")

    private weak var _delegate: TunnelMonitorDelegate?
    weak var delegate: TunnelMonitorDelegate? {
        set {
            nslock.lock()
            defer { nslock.unlock() }

            _delegate = newValue
        }
        get {
            nslock.lock()
            defer { nslock.unlock() }

            return _delegate
        }
    }

    init(
        delegateQueue: DispatchQueue,
        packetTunnelProvider: NEPacketTunnelProvider,
        adapter: WireGuardAdapter
    ) {
        self.delegateQueue = delegateQueue
        self.packetTunnelProvider = packetTunnelProvider
        self.adapter = adapter

        pinger = Pinger(delegateQueue: eventQueue)
        pinger.delegate = self
    }

    deinit {
        stop()
    }

    func start(probeAddress: IPv4Address) {
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

    func stop() {
        nslock.lock()
        defer { nslock.unlock() }

        _stop()
    }

    func onWake() {
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

    func onSleep(completion: @escaping () -> Void) {
        nslock.lock()
        defer { nslock.unlock() }

        logger.trace("Prepare to sleep.")

        stopConnectivityCheckTimer()
        removeDefaultPathObserver()
    }

    // MARK: - PingerDelegate

    func pinger(
        _ pinger: Pinger,
        didReceiveResponseFromSender senderAddress: IPAddress,
        icmpHeader: ICMPHeader
    ) {
        didReceivePing(from: senderAddress, icmpHeader: icmpHeader)
    }

    func pinger(_ pinger: Pinger, didFailWithError error: Error) {
        logger.error(
            error: error,
            message: "Failed to parse ICMP response."
        )
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
        guard let packetTunnelProvider else { return }

        defaultPathObserver?.invalidate()

        logger.trace("Add default path observer.")

        defaultPathObserver = packetTunnelProvider
            .observe(\.defaultPath, options: [.new]) { [weak self] _, change in
                guard let self else { return }

                nslock.lock()
                defer { self.nslock.unlock() }

                let newValue = change.newValue.flatMap { $0 }
                if let newPath = newValue {
                    handleNetworkPathUpdate(newPath)
                }
            }

        if let currentPath = packetTunnelProvider.defaultPath {
            handleNetworkPathUpdate(currentPath)
        }
    }

    private func removeDefaultPathObserver() {
        guard let defaultPathObserver else { return }

        logger.trace("Remove default path observer.")

        defaultPathObserver.invalidate()
        self.defaultPathObserver = nil
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

        sendDelegateShouldHandleConnectionRecovery()
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

    private func handleNetworkPathUpdate(_ networkPath: NetworkExtension.NWPath) {
        let pathStatus = networkPath.status
        let isReachable = pathStatus == .satisfiable || pathStatus == .satisfied

        switch state.connectionState {
        case .pendingStart:
            if isReachable {
                logger.debug("Start monitoring connection.")
                startMonitoring()
                sendDelegateNetworkStatusChange(true)
            } else {
                logger.debug("Wait for network to become reachable before starting monitoring.")
                state.connectionState = .waitingConnectivity
                sendDelegateNetworkStatusChange(false)
            }

        case .waitingConnectivity:
            guard isReachable else { return }

            logger.debug("Network is reachable. Resume monitoring.")
            startMonitoring()
            sendDelegateNetworkStatusChange(true)

        case .connecting, .connected:
            guard !isReachable else { return }

            logger.debug("Network is unreachable. Pause monitoring.")
            state.connectionState = .waitingConnectivity
            stopMonitoring(resetRetryAttempt: true)
            sendDelegateNetworkStatusChange(false)

        case .stopped, .recovering:
            break
        }
    }

    private func didReceivePing(from sender: IPAddress, icmpHeader: ICMPHeader) {
        nslock.lock()
        defer { nslock.unlock() }

        guard let probeAddress else { return }

        if sender.rawValue != probeAddress.rawValue {
            logger.trace("Got reply from unknown sender: \(sender), expected: \(probeAddress).")
        }

        let now = Date()
        let sequenceNumber = icmpHeader.sequenceNumber
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
            sendDelegateConnectionEstablished()
        }
    }

    private func startMonitoring() {
        do {
            guard let interfaceName = adapter.interfaceName else {
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
        let timer = DispatchSource.makeTimerSource(queue: eventQueue)
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

    private func sendDelegateConnectionEstablished() {
        delegateQueue.async {
            self.delegate?.tunnelMonitorDidDetermineConnectionEstablished(self)
        }
    }

    private func sendDelegateShouldHandleConnectionRecovery() {
        delegateQueue.async {
            self.delegate?.tunnelMonitorDelegateShouldHandleConnectionRecovery(self)
        }
    }

    private func sendDelegateNetworkStatusChange(_ isNetworkReachable: Bool) {
        delegateQueue.async {
            self.delegate?.tunnelMonitor(
                self,
                networkReachabilityStatusDidChange: isNetworkReachable
            )
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
        var result: String?

        let dispatchGroup = DispatchGroup()
        dispatchGroup.enter()
        adapter.getRuntimeConfiguration { string in
            result = string
            dispatchGroup.leave()
        }

        guard case .success = dispatchGroup.wait(wallTimeout: .now() + .seconds(1)) else {
            logger.debug("adapter.getRuntimeConfiguration timeout.")
            return nil
        }

        guard let result else {
            logger.debug("Received nil string for stats.")
            return nil
        }

        guard let newStats = WgStats(from: result) else {
            logger.debug("Couldn't parse stats.")
            return nil
        }

        return newStats
    }
}
