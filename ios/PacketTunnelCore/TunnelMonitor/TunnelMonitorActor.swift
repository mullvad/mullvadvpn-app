//
//  TunnelMonitorActor.swift
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

public actor TunnelMonitorActor: TunnelMonitorActorProtocol {
    private let pinger: any PingerProtocol
    private let timings: TunnelMonitorTimings
    private var state: TunnelMonitorState
    private let tunnelDeviceInfo: any TunnelDeviceInfoProtocol
    private var probeAddress: IPv4Address?
    private var timer: DispatchSourceTimer?
    private var eventHandler: AsyncStream<TunnelMonitorEvent>.Continuation?

    public let eventStream: AsyncStream<TunnelMonitorEvent>

    init(
        pinger: any PingerProtocol,
        timings: TunnelMonitorTimings,
        tunnelDeviceInfo: any TunnelDeviceInfoProtocol
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

    public func start(probeAddress: IPv4Address) async throws {
        print(#function)
        guard state.connectionState == .stopped else { return }

        self.probeAddress = probeAddress
        self.state.connectionState = .pendingStart
        pinger.onReply = { [weak self] reply in
            Task {
                await self?.handlePingerReply(reply)
            }
        }

        try startMonitoring()
    }

    public func stop() async {
        print(#function)
        _stop()
    }

    public func wake() async {
        print(#function)
    }

    public func sleep() async {
        print(#function)
    }

    public func handleNetworkPathUpdate(_ networkPath: any NetworkPath) async {
        print(#function)

        let pathStatus = networkPath.status
        let isReachable = pathStatus == .satisfiable || pathStatus == .satisfied

        switch state.connectionState {
        case .pendingStart:
            if isReachable {
                try? startMonitoring()
            } else {
                state.connectionState = .waitingConnectivity
            }

        case .waitingConnectivity:
            guard isReachable else { return }

            try? startMonitoring()

        case .connecting, .connected:
            guard !isReachable else { return }

            state.connectionState = .waitingConnectivity
            stopMonitoring(resetRetryAttempt: true)

        case .stopped, .recovering:
            break
        }
    }

    // MARK: - Private API

    private func _stop(restarting: Bool = false) {
        guard state.connectionState != .stopped else { return }

        probeAddress = nil
        stopMonitoring(resetRetryAttempt: restarting == false)
        state.connectionState = .stopped
    }

    // TODO: Should this be marked `throws` ?
    private func startMonitoring() throws {
        guard let probeAddress else { return }

        try pinger.startPinging(destAddress: probeAddress)
        state.connectionState = .connecting
        startConnectivityCheckTimer()
    }

    private func stopMonitoring(resetRetryAttempt: Bool) {
        stopConnectivityCheckTimer()
        pinger.stopPinging()
        state.reset(resetRetryAttempts: resetRetryAttempt)
    }

    private func handlePingerReply(_ reply: PingerReply) {}

    private func checkConnectivity() {
        guard let newStats = try? tunnelDeviceInfo.getStats(),
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

        guard timer?.isCancelled == false else { return }

        let timerSource = DispatchSource.makeTimerSource()
        timerSource.setEventHandler(handler: checkConnectivity)
        timerSource.schedule(wallDeadline: .now(), repeating: timings.connectivityCheckInterval.timeInterval)
        timerSource.activate()

        timer?.cancel()
        timer = timerSource

        state.timeoutReference = Date()
    }

    private func stopConnectivityCheckTimer() {
        print(#function)
    }
}
