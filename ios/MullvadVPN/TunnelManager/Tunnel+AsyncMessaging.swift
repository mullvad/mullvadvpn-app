//
//  Tunnel+AsyncMessaging.swift
//  MullvadVPN
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import NetworkExtension
import PacketTunnelCore

/// Delay for sending tunnel provider messages to the tunnel when in connecting state.
/// Used to workaround a bug when talking to the tunnel too early during startup may cause it
/// to freeze.
private let connectingStateWaitDelay: Duration = .seconds(5)

/// Default timeout in seconds.
private let defaultTimeout: Duration = .seconds(5)

/// Async counterparts to the operation-based IPC in `Tunnel+Messaging.swift`.
/// New code should prefer these; the operation-based variants remain until all
/// message types are migrated.
extension TunnelProtocol {
    /// Request packet tunnel process to reconnect the tunnel with the given relays.
    func reconnectTunnel(to nextRelays: NextRelays) async throws -> ObservedState {
        let responseData = try await sendProviderMessage(.reconnectTunnel(nextRelays))
        return try mapObservedState(data: responseData)
    }

    /// Send a message to the packet tunnel process and await the reply.
    ///
    /// Mirrors `SendTunnelProviderMessageOperation` semantics: the message is sent once the
    /// tunnel reports `.connected` or `.reasserting`. In `.connecting`, the send is delayed
    /// until `connectingStateWaitDelay` has passed since the tunnel was started, unless the
    /// status changes earlier. Down states fail with `tunnelDown`. The timeout window is
    /// extended by the remaining connecting-state delay, matching the legacy timer reschedule.
    func sendProviderMessage(
        _ message: TunnelProviderMessage,
        timeout: Duration = defaultTimeout
    ) async throws -> Data? {
        let messageData = try message.encode()
        let timeoutWindow =
            status == .connecting
            ? timeout + remainingConnectingStateDelay.timeInterval
            : timeout

        return try await withThrowingTaskGroup(of: Data?.self) { group in
            group.addTask {
                try await self.waitForActiveStateAndSend(messageData, timeout: timeout)
            }
            group.addTask {
                try await Task.sleep(for: timeoutWindow)
                throw SendTunnelProviderMessageError.timeout
            }

            defer { group.cancelAll() }
            return try await group.next()!
        }
    }

    /// Drive the status-dependent send loop: react to every status change until the message
    /// is sent or the tunnel goes down.
    private func waitForActiveStateAndSend(_ messageData: Data, timeout: Duration) async throws -> Data? {
        var streamContinuation: AsyncStream<Event>.Continuation!
        let events = AsyncStream<Event> { streamContinuation = $0 }
        let continuation = streamContinuation!

        let observer = addBlockObserver(queue: nil) { _, status in
            continuation.yield(.status(status))
        }
        var connectingDelayTask: Task<Void, Never>?
        defer {
            connectingDelayTask?.cancel()
            observer.invalidate()
            continuation.finish()
        }

        continuation.yield(.status(status))

        for await event in events {
            switch event {
            case let .status(vpnStatus):
                switch vpnStatus {
                case .connected, .reasserting:
                    return try await send(messageData, timeout: timeout)

                case .connecting:
                    guard connectingDelayTask == nil else { break }

                    let remainingDelay = remainingConnectingStateDelay
                    guard remainingDelay > .zero else {
                        return try await send(messageData, timeout: timeout)
                    }

                    connectingDelayTask = Task {
                        try? await Task.sleep(for: remainingDelay)
                        guard !Task.isCancelled else { return }
                        continuation.yield(.connectingDelayElapsed)
                    }

                case .invalid, .disconnecting, .disconnected:
                    throw SendTunnelProviderMessageError.tunnelDown(vpnStatus)

                @unknown default:
                    break
                }

            case .connectingDelayElapsed:
                return try await send(messageData, timeout: timeout)
            }
        }

        // The stream ends early when the surrounding task is cancelled (e.g. by the timeout
        // racing task or by the caller).
        throw CancellationError()
    }

    private func send(_ messageData: Data, timeout: Duration) async throws -> Data? {
        guard backgroundTaskProvider.backgroundTimeRemaining > timeout else {
            throw SendTunnelProviderMessageError.notEnoughBackgroundTime
        }

        let resumer = SingleResumer<Data?>()

        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                resumer.setContinuation(continuation)

                do {
                    try sendProviderMessage(messageData) { responseData in
                        resumer.resume(with: .success(responseData))
                    }
                } catch {
                    resumer.resume(with: .failure(SendTunnelProviderMessageError.system(error)))
                }
            }
        } onCancel: {
            // NetworkExtension offers no way to cancel an in-flight provider message, so
            // cancellation abandons the eventual response.
            resumer.resume(with: .failure(CancellationError()))
        }
    }

    /// Remaining portion of `connectingStateWaitDelay` measured from the tunnel start date.
    private var remainingConnectingStateDelay: Duration {
        let timeElapsed: TimeInterval = startDate.map { Date().timeIntervalSince($0) } ?? 0
        guard timeElapsed < connectingStateWaitDelay else {
            return .zero
        }
        return connectingStateWaitDelay - timeElapsed
    }
}

/// Events driving the status-dependent send loop in `waitForActiveStateAndSend`.
private enum Event: Sendable {
    case status(NEVPNStatus)
    case connectingDelayElapsed
}

/// Resumes a checked continuation exactly once, no matter how the response handler and the
/// cancellation handler race. Also tolerates a result arriving before the continuation is set
/// (e.g. cancellation fired before suspension). The first result wins; later ones are dropped.
final class SingleResumer<Value: Sendable>: @unchecked Sendable {
    private let lock = NSLock()
    private var continuation: CheckedContinuation<Value, Error>?
    private var pendingResult: Result<Value, Error>?

    func setContinuation(_ newContinuation: CheckedContinuation<Value, Error>) {
        lock.lock()
        if let result = pendingResult {
            pendingResult = nil
            lock.unlock()
            newContinuation.resume(with: result)
        } else {
            continuation = newContinuation
            lock.unlock()
        }
    }

    func resume(with result: Result<Value, Error>) {
        lock.lock()
        if let resumableContinuation = continuation {
            continuation = nil
            lock.unlock()
            resumableContinuation.resume(with: result)
        } else {
            if pendingResult == nil {
                pendingResult = result
            }
            lock.unlock()
        }
    }
}
