//
//  Tunnel+AsyncMessaging.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadREST
import MullvadTypes
import NetworkExtension
import PacketTunnelCore

/// Delay for sending tunnel provider messages to the tunnel when in connecting state.
///
/// Used to work around a bug where talking to the tunnel too early during
/// startup may cause it to freeze.
private let connectingStateWaitDelay: Duration = .seconds(5)

/// Default timeout.
private let defaultTimeout: Duration = .seconds(5)

extension TunnelProtocol {

    /// Request packet tunnel process to reconnect the tunnel with the given relays.
    func reconnectTunnel(to nextRelays: NextRelays) async throws -> ObservedState {
        let result = try await sendProviderMessage(.reconnectTunnel(nextRelays))
        return try mapObservedState(data: result)
    }

    /// Request status from packet tunnel process.
    func getTunnelStatus() async throws -> ObservedState {
        let result = try await sendProviderMessage(.getTunnelStatus)
        return try mapObservedState(data: result)
    }

    func mapObservedState(data: Data?) throws -> ObservedState {
        if let data {
            return try TunnelProviderReply<ObservedState>(messageData: data).value
        } else {
            throw EmptyTunnelProviderResponseError()
        }
    }
    
    private func sendProviderMessage(
        _ message: TunnelProviderMessage,
        timeout: Duration = defaultTimeout
    ) async throws
        -> Data?
    {
        try Task.checkCancellation()
        let messageData = try message.encode()
        let remainingDelay = await remainingConnectingStateDelay()
        let timeoutWindow =
            status == .connecting
            ? timeout + remainingDelay
            : timeout

        return try await withThrowingTaskGroup(of: Data?.self) { group in
            group.addTask {
                try await self.waitingForTunnelStatus(messageData, timeout: timeout)
            }
            group.addTask {
                try await Task.sleep(for: timeoutWindow)
                throw SendTunnelProviderMessageError.timeout
            }

            defer { group.cancelAll() }
            return try await group.next()!
        }
    }

    private func handleVPNStatus(_ message: Data, timeout: Duration) async throws -> Data? {
        try Task.checkCancellation()
        switch status {
        case .connected, .reasserting:
            return try await send(message, timeout: timeout)
        case .connecting:
            let remainingDelay = await remainingConnectingStateDelay()
            guard remainingDelay > .zero else {
                return try await send(message, timeout: timeout)
            }
            try await Task.sleep(for: remainingDelay)
            try Task.checkCancellation()
            return try await send(message, timeout: timeout)

        case .invalid, .disconnecting, .disconnected:
            throw SendTunnelProviderMessageError.tunnelDown(status)

        @unknown default:
            break

        }
        throw CancellationError()
    }

    private func send(_ messageData: Data, timeout: Duration) async throws -> Data? {
        try Task.checkCancellation()
        guard backgroundTaskProvider.backgroundTimeRemaining > timeout else {
            throw SendTunnelProviderMessageError.notEnoughBackgroundTime
        }
        return try await withCheckedThrowingContinuation { continuation in
            do {
                try sendProviderMessage(messageData) { data in
                    continuation.resume(returning: data)
                }

            } catch {
                continuation.resume(throwing: error)
            }
        }
    }

    private func waitingForTunnelStatus(_ message: Data, timeout: Duration) async throws -> Data? {
        var asyncStream: AsyncStream<NEVPNStatus>.Continuation!
        let events = AsyncStream<NEVPNStatus> { asyncStream = $0 }

        let observer = self.addBlockObserver(
            queue: .main,
            handler: { _, tunnelStatus in
                asyncStream.yield(tunnelStatus)
            })

        defer {
            asyncStream.finish()
            removeObserver(observer)
        }

        for await event in events {
            return try await handleVPNStatus(message, timeout: timeout)
        }
        return nil
    }

    /// Remaining portion of `connectingStateWaitDelay` measured from the tunnel start date.
    private func remainingConnectingStateDelay() async -> Duration {
        let timeElapsed: Duration

        if let startDate = await startDate {
            timeElapsed = .seconds(Date().timeIntervalSince(startDate))
        } else {
            timeElapsed = .zero
        }

        guard timeElapsed < connectingStateWaitDelay else {
            return .zero
        }

        return connectingStateWaitDelay - timeElapsed
    }
}
