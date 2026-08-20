//
//  SendTunnelProviderMessageTask.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import NetworkExtension
import PacketTunnelCore

final class SendTunnelProviderMessageTask: Sendable {
    private let tunnel: any TunnelProtocol
    private let message: TunnelProviderMessage
    private let timeout: Duration

    /// Delay for sending tunnel provider messages to the tunnel when in connecting state.
    ///
    /// Used to work around a bug where talking to the tunnel too early during
    /// startup may cause it to freeze.
    private let connectingStateWaitDelay: Duration = .seconds(5)

    /// Default timeout.
    private let defaultTimeout: Duration = .seconds(5)

    init(
        tunnel: any TunnelProtocol,
        message: TunnelProviderMessage,
        timeout: Duration? = nil
    ) {
        self.tunnel = tunnel
        self.message = message
        self.timeout = timeout ?? defaultTimeout
    }

    func start() async throws -> Data? {
        try await waitForConnectingState()
    }

    private func waitForConnectingState() async throws -> Data? {
        var streamContinuation: AsyncStream<NEVPNStatus>.Continuation!

        let events = AsyncStream<NEVPNStatus> {
            streamContinuation = $0
        }

        let observer = tunnel.addBlockObserver(queue: nil) { _, status in
            streamContinuation.yield(status)
        }

        defer {
            streamContinuation.finish()
            observer.invalidate()
        }

        streamContinuation.yield(tunnel.status)

        for await status in events {
            return try await handleVPNStatus(status)
        }

        return nil
    }

    private func handleVPNStatus(
        _ status: NEVPNStatus
    ) async throws -> Data? {
        switch status {
        case .connected, .reasserting:
            return try await sendMessage()

        case .connecting:
            let timeElapsed: Duration

            if let startDate = await tunnel.startDate {
                timeElapsed = .seconds(
                    Date().timeIntervalSince(startDate)
                )
            } else {
                timeElapsed = .zero
            }

            if timeElapsed < connectingStateWaitDelay {
                try await Task.sleep(
                    for: connectingStateWaitDelay - timeElapsed
                )
            }

            return try await sendMessage()

        case .invalid, .disconnecting, .disconnected:
            throw SendTunnelProviderMessageError.tunnelDown(status)

        default:
            return nil
        }
    }

    private func sendMessage() async throws -> Data? {
        try Task.checkCancellation()

        let messageData = try message.encode()
        let state = ContinuationState()
//        let bridge = ContinuationBridge<Data?>()

//        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in
                guard !Task.isCancelled else {
                    return continuation.resume(returning: .failure(CancellationError()))
                }
  
                bridge.set(continuation)

                do {
                    try tunnel.sendProviderMessage(messageData) { responseData in
                        bridge.resume(with: .success(responseData))
                    }
                } catch {
                    bridge.resume(with: .failure(error))
                }
            }
//        }
//        onCancel: {
//            Task {
//                state.cancel()
//            }
////            bridge.resume(with: .failure(CancellationError()))
//        }
    }
}
actor ContinuationState{
    private var isCanceled=false
    func cancel() {
        isCanceled = true
    }
    func isRunning()->Bool{
       isCanceled = false
    }
}

//private final class ContinuationBridge<Value: Sendable>: @unchecked Sendable {
//    private let lock = NSLock()
//
//    private var continuation: CheckedContinuation<Value, Error>?
//    private var pendingResult: Result<Value, Error>?
//
//    func set(
//        _ continuation: CheckedContinuation<Value, Error>
//    ) {
//        lock.lock()
//
//        if let result = pendingResult {
//            pendingResult = nil
//            lock.unlock()
//
//            continuation.resume(with: result)
//            return
//        }
//
//        self.continuation = continuation
//        lock.unlock()
//    }
//
//    func resume(with result: Result<Value, Error>) {
//        lock.lock()
//
//        guard pendingResult == nil else {
//            lock.unlock()
//            return
//        }
//
//        if let continuation {
//            self.continuation = nil
//            lock.unlock()
//
//            continuation.resume(with: result)
//        } else {
//            pendingResult = result
//            lock.unlock()
//        }
//    }
//}
