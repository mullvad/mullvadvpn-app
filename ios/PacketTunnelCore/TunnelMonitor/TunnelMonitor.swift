//
//  TunnelMonitor.swift
//  PacketTunnelCore
//
//  Created by pronebird on 09/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Network
import NetworkExtension

/// Tunnel monitor.
public final class TunnelMonitor: TunnelMonitorProtocol {
    private let tunnelDeviceInfo: TunnelDeviceInfoProtocol

    private let nslock = NSLock()
    private let timerQueue = DispatchQueue(label: "TunnelMonitor-timerQueue")
    private let eventQueue: DispatchQueue
    private let timings: TunnelMonitorTimings

    private var pinger: PingerProtocol
    private var isObservingDefaultPath = false
    private var timer: DispatchSourceTimer?

    private var state: TunnelMonitorState
    private var probeAddress: IPv4Address?

    private let logger = Logger(label: "TunnelMonitor")

    private var _onEvent: ((TunnelMonitorEvent) -> Void)?
    public var onEvent: ((TunnelMonitorEvent) -> Void)? {
        get {
            nslock.withLock {
                return _onEvent
            }
        }
        set {
            nslock.withLock {
                _onEvent = newValue
            }
        }
    }

    public init(
        eventQueue: DispatchQueue,
        pinger: PingerProtocol,
        tunnelDeviceInfo: TunnelDeviceInfoProtocol,
        timings: TunnelMonitorTimings
    ) {
        self.eventQueue = eventQueue
        self.tunnelDeviceInfo = tunnelDeviceInfo

        self.timings = timings
        state = TunnelMonitorState(timings: timings)

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

        startMonitoring()
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

        case .waitingConnectivity, .pendingStart, .stopped, .recovering:
            break
        }
    }

    public func onSleep() {
        nslock.lock()
        defer { nslock.unlock() }

        logger.trace("Prepare to sleep.")

        stopConnectivityCheckTimer()
    }

    public func handleNetworkPathUpdate(_ networkPath: NetworkPath) {
        nslock.withLock {
            let pathStatus = networkPath.status
            let isReachable = pathStatus == .satisfiable || pathStatus == .satisfied

            switch state.connectionState {
            case .pendingStart:
                if isReachable {
                    logger.debug("Start monitoring connection.")
                    startMonitoring()
                } else {
                    logger.debug("Wait for network to become reachable before starting monitoring.")
                    state.connectionState = .waitingConnectivity
                }

            case .waitingConnectivity:
                guard isReachable else { return }

                logger.debug("Network is reachable. Resume monitoring.")
                startMonitoring()

            case .connecting, .connected:
                guard !isReachable else { return }

                logger.debug("Network is unreachable. Pause monitoring.")
                state.connectionState = .waitingConnectivity
                stopMonitoring(resetRetryAttempt: true)

            case .stopped, .recovering:
                break
            }
        }
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

        stopMonitoring(resetRetryAttempt: !forRestart)

        state.connectionState = .stopped
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
        timer.schedule(wallDeadline: .now(), repeating: timings.connectivityCheckInterval.timeInterval)
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

    private func getStats() -> WgStats? {
        do {
            return try tunnelDeviceInfo.getStats()
        } catch {
            logger.error(error: error, message: "Failed to obtain adapter stats.")

            return nil
        }
    }
}
