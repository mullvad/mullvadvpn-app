//
//  TunnelMonitor.swift
//  PacketTunnelCore
//
//  Created by Marco Nikic on 2025-01-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import Network
import NetworkExtension

public actor TunnelMonitorActor: TunnelMonitorProtocol {
    private let pinger: any PingerProtocol
    private let timings: TunnelMonitorTimings
    private var state: TunnelMonitorState
    private let tunnelDeviceInfo: any TunnelDeviceInfoProtocol
    private var probeAddress: IPv4Address?
    private var timer: DispatchSourceTimer?
    private var eventHandler: AsyncStream<TunnelMonitorEvent>.Continuation?

    public let eventStream: AsyncStream<TunnelMonitorEvent>

    public init(
        pinger: any PingerProtocol,
        tunnelDeviceInfo: any TunnelDeviceInfoProtocol,
        timings: TunnelMonitorTimings
    ) {
        self.pinger = pinger
        self.timings = timings
        self.state = TunnelMonitorState(timings: timings)
        self.tunnelDeviceInfo = tunnelDeviceInfo

        var innerContinuation: AsyncStream<TunnelMonitorEvent>.Continuation?
        let stream = AsyncStream<TunnelMonitorEvent> { continuation in
            innerContinuation = continuation
        }

        self.eventStream = stream
        self.eventHandler = innerContinuation
    }

    // MARK: - Public API

    public func start(probeAddress: IPv4Address) async {
        if case .stopped = state.connectionState {
            print("Start with address: \(probeAddress).")
        } else {
            _stop(restarting: true)
        }

        self.probeAddress = probeAddress
        self.state.connectionState = .pendingStart

        pinger.onReply = { @Sendable [weak self] reply in
            Task {
                await self?.handlePingerReply(reply)
            }
        }

        startMonitoring()
    }

    public func stop() async {
        _stop()
    }

    private func _stop(restarting: Bool = false) {
        guard state.connectionState != .stopped else { return }

        pinger.onReply = nil

        probeAddress = nil
        stopMonitoring(resetRetryAttempt: restarting == false)
        state.connectionState = .stopped
    }

    public func wake() async {
        switch state.connectionState {
        case .connecting, .connected:
            startConnectivityCheckTimer()

        case .waitingConnectivity, .pendingStart, .stopped, .recovering:
            break
        }
    }

    public func sleep() async {
        print(#function)
        stopConnectivityCheckTimer()
    }

    public func handleNetworkPathUpdate(_ networkPath: Network.NWPath.Status) async {
        print(#function)

        let pathStatus = networkPath
        let isReachable = pathStatus == .satisfied || pathStatus == .requiresConnection
        let message = "handleNetworkPathUpdate considered reachable: \(isReachable)"
        print(message)

        switch state.connectionState {
        case .pendingStart:
            if isReachable {
                startMonitoring()
            } else {
                state.connectionState = .waitingConnectivity
            }

        case .waitingConnectivity:
            guard isReachable else { return }

            startMonitoring()

        case .connecting, .connected:
            guard !isReachable else { return }

            state.connectionState = .waitingConnectivity
            stopMonitoring(resetRetryAttempt: true)

        case .stopped, .recovering:
            break
        }
    }

    // MARK: - Private API

    private func startMonitoring() {
        guard let probeAddress else { return }

        pinger.startPinging(destAddress: probeAddress)
        state.connectionState = .connecting
        startConnectivityCheckTimer()
    }

    private func stopMonitoring(resetRetryAttempt: Bool) {
        stopConnectivityCheckTimer()
        pinger.stopPinging()
        state.reset(resetRetryAttempts: resetRetryAttempt)
    }

    private func handlePingerReply(_ reply: PingerReply) {
        switch reply {
        case let .success(sender, sequenceNumber):
            guard let probeAddress else { return }

            if sender.rawValue != probeAddress.rawValue {
                print("Got reply from unknown sender: \(sender), expected: \(probeAddress).")
            }

            let now = Date()
            guard state.setPingReplyReceived(sequenceNumber, now: now) != nil else {
                print("Got unknown ping sequence: \(sequenceNumber).")
                return
            }

            if case .connecting = state.connectionState {
                state.connectionState = .connected
                state.retryAttempt = 0
                sendConnectionEstablishedEvent()
            }

        case let .parseError(error):
            print("Failed to parse ICMP response: \(error)")
        }
    }

    private func checkConnectivity() async {
        print(#function)
        let newStats = try? await tunnelDeviceInfo.getStats()
        let statsDebug = "bytes received: \(newStats?.bytesReceived), bytes sent: \(newStats?.bytesSent)"
        print(statsDebug)
        guard let newStats,
              state.connectionState == .connecting || state.connectionState == .connected
        else { return }

        // Check if counters were reset.
        let isStatsReset = newStats.bytesReceived < state.netStats.bytesReceived ||
            newStats.bytesSent < state.netStats.bytesSent

        guard !isStatsReset else {
            state.netStats = newStats
            return
        }

        let now = Date()
        state.updateNetStats(newStats: newStats, now: now)

        let timeout = state.getPingTimeout()
        let expectedTimeout = "Expected timeout is \(timeout)"
        print(expectedTimeout)
        let evaluation = state.evaluateConnection(now: now, pingTimeout: timeout)

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
            sendPing(now: now)
        }
    }

    private func sendPing(now: Date) {
        do {
            let sendResult = try pinger.send()
            state.updatePingStats(sendResult: sendResult, now: now)
        } catch {
            print("Failed to send ping.")
        }
    }

    private func sendConnectionEstablishedEvent() {
        eventHandler?.yield(.connectionEstablished)
    }

    private func sendConnectionLostEvent() {
        eventHandler?.yield(.connectionLost)
    }

    private func startConnectionRecovery() {
        stopMonitoring(resetRetryAttempt: false)

        state.retryAttempt = state.retryAttempt.saturatingAddition(1)
        state.connectionState = .recovering
        probeAddress = nil

        sendConnectionLostEvent()
    }

    private func startConnectivityCheckTimer() {
        print(#function)

        let timerSource = DispatchSource.makeTimerSource()

        timerSource.setEventHandler {
            Task { [weak self] in
                await self?.checkConnectivity()
            }
        }

        timerSource.schedule(wallDeadline: .now(), repeating: timings.connectivityCheckInterval.timeInterval)
        timerSource.activate()

        timer?.cancel()
        timer = timerSource

        state.timeoutReference = Date()
    }

    private func stopConnectivityCheckTimer() {
        print(#function)
        timer?.setEventHandler(
            handler: {}
        )
        timer?.cancel()
        timer = nil
    }

    #if DEBUG
    internal func getState() -> TunnelMonitorState {
        TunnelMonitorState(
            connectionState: state.connectionState,
            netStats: state.netStats,
            pingStats: state.pingStats,
            timeoutReference: state.timeoutReference,
            lastSeenRx: state.lastSeenRx,
            lastSeenTx: state.lastSeenTx,
            isHeartbeatSuspended: state.isHeartbeatSuspended,
            retryAttempt: state.retryAttempt,
            timings: timings
        )
    }
    #endif
}
