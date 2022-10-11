//
//  SendTunnelProviderMessageOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import Operations

private enum MessagingConfiguration {
    /// Delay for sending tunnel provider messages to the tunnel when in connecting state.
    /// Used to workaround a bug when talking to the tunnel too early during startup may cause it
    /// to freeze.
    static let connectingStateWaitDelay: TimeInterval = 5

    /// Timeout interval in seconds.
    static let timeout: TimeInterval = 5
}

final class SendTunnelProviderMessageOperation<Output>: ResultOperation<Output, Error> {
    typealias DecoderHandler = (Data?) throws -> Output

    private let tunnel: Tunnel
    private let message: TunnelProviderMessage
    private let decoderHandler: DecoderHandler

    private var statusObserver: TunnelStatusBlockObserver?
    private var timeoutWork: DispatchWorkItem?
    private var waitForConnectingStateWork: DispatchWorkItem?

    private var messageSent = false

    init(
        dispatchQueue: DispatchQueue,
        tunnel: Tunnel,
        message: TunnelProviderMessage,
        decoderHandler: @escaping DecoderHandler,
        completionHandler: @escaping CompletionHandler
    ) {
        self.tunnel = tunnel
        self.message = message
        self.decoderHandler = decoderHandler

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        setTimeoutTimer(connectingStateWaitDelay: 15)

        statusObserver = tunnel.addBlockObserver(queue: dispatchQueue) { [weak self] _, status in
            self?.handleVPNStatus(status)
        }

        handleVPNStatus(tunnel.status)
    }

    override func operationDidCancel() {
        if isExecuting {
            finish(completion: .cancelled)
        }
    }

    override func finish(completion: Completion) {
        // Release status observer.
        removeVPNStatusObserver()

        // Cancel pending work.
        timeoutWork?.cancel()
        waitForConnectingStateWork?.cancel()

        // Finish operation.
        super.finish(completion: completion)
    }

    private func removeVPNStatusObserver() {
        statusObserver?.invalidate()
        statusObserver = nil
    }

    private func setTimeoutTimer(connectingStateWaitDelay delay: TimeInterval) {
        let workItem = DispatchWorkItem { [weak self] in
            self?.finish(completion: .failure(SendTunnelProviderMessageError.timeout))
        }

        // Cancel pending timeout work.
        timeoutWork?.cancel()

        // Assign new timeout work.
        timeoutWork = workItem

        // Schedule timeout work.
        let deadline: DispatchWallTime = .now() + MessagingConfiguration.timeout + delay

        dispatchQueue.asyncAfter(wallDeadline: deadline, execute: workItem)
    }

    private func handleVPNStatus(_ status: NEVPNStatus) {
        guard !isCancelled, !messageSent else {
            return
        }

        switch status {
        case .connected:
            sendMessage()

        case .connecting:
            waitForConnectingState { [weak self] in
                self?.sendMessage()
            }

        case .reasserting:
            sendMessage()
        case .invalid, .disconnecting, .disconnected:
            finish(completion: .failure(SendTunnelProviderMessageError.tunnelDown(status)))

        @unknown default:
            break
        }
    }

    private func waitForConnectingState(block: @escaping () -> Void) {
        // Compute amount of time elapsed since the tunnel was launched.
        let timeElapsed: TimeInterval
        if let startDate = tunnel.startDate {
            timeElapsed = Date().timeIntervalSince(startDate)
        } else {
            timeElapsed = 0
        }

        // Cancel pending work.
        waitForConnectingStateWork?.cancel()
        waitForConnectingStateWork = nil

        // Execute right away if enough time passed since the tunnel was launched.
        guard timeElapsed < MessagingConfiguration.connectingStateWaitDelay else {
            block()
            return
        }

        let waitDelay = MessagingConfiguration.connectingStateWaitDelay - timeElapsed
        let workItem = DispatchWorkItem(block: block)

        // Assign new work.
        waitForConnectingStateWork = workItem

        // Reschedule the timeout work.
        setTimeoutTimer(connectingStateWaitDelay: waitDelay)

        // Schedule delayed work.
        let deadline: DispatchWallTime = .now() + waitDelay

        dispatchQueue.asyncAfter(wallDeadline: deadline, execute: workItem)
    }

    private func sendMessage() {
        // Mark message sent.
        messageSent = true

        // Release status observer.
        removeVPNStatusObserver()

        // Cancel pending delayed work.
        waitForConnectingStateWork?.cancel()

        // Encode message.
        let messageData: Data
        do {
            messageData = try message.encode()
        } catch {
            finish(completion: .failure(error))
            return
        }

        // Send IPC message.
        do {
            try tunnel.sendProviderMessage(messageData) { [weak self] responseData in
                guard let self = self else { return }

                self.dispatchQueue.async {
                    let decodingResult = Result { try self.decoderHandler(responseData) }

                    self.finish(completion: OperationCompletion(result: decodingResult))
                }
            }
        } catch {
            finish(completion: .failure(SendTunnelProviderMessageError.system(error)))
        }
    }
}

extension SendTunnelProviderMessageOperation where Output: Codable {
    convenience init(
        dispatchQueue: DispatchQueue,
        tunnel: Tunnel,
        message: TunnelProviderMessage,
        completionHandler: @escaping CompletionHandler
    ) {
        self.init(
            dispatchQueue: dispatchQueue,
            tunnel: tunnel,
            message: message,
            decoderHandler: { data in
                if let data = data {
                    return try TunnelProviderReply(messageData: data).value
                } else {
                    throw EmptyTunnelProviderResponseError()
                }
            },
            completionHandler: completionHandler
        )
    }
}

/// Separating TransportMessageReply from Codable for future use. (Logs, separate strategies, etc)
extension SendTunnelProviderMessageOperation where Output == TransportMessageReply {
    convenience init(
        dispatchQueue: DispatchQueue,
        tunnel: Tunnel,
        message: TunnelProviderMessage,
        completionHandler: @escaping CompletionHandler
    ) {
        self.init(
            dispatchQueue: dispatchQueue,
            tunnel: tunnel,
            message: message,
            decoderHandler: { data in
                if let data = data {
                    return try TunnelProviderReply(messageData: data).value
                } else {
                    throw EmptyTunnelProviderResponseError()
                }
            },
            completionHandler: completionHandler
        )
    }
}

extension SendTunnelProviderMessageOperation where Output == Void {
    convenience init(
        dispatchQueue: DispatchQueue,
        tunnel: Tunnel,
        message: TunnelProviderMessage,
        completionHandler: @escaping CompletionHandler
    ) {
        self.init(
            dispatchQueue: dispatchQueue,
            tunnel: tunnel,
            message: message,
            decoderHandler: { _ in () },
            completionHandler: completionHandler
        )
    }
}

enum SendTunnelProviderMessageError: LocalizedError, WrappingError {
    /// Tunnel process is either down or about to go down.
    case tunnelDown(NEVPNStatus)

    /// Timeout.
    case timeout

    /// System error.
    case system(Swift.Error)

    var errorDescription: String? {
        switch self {
        case let .tunnelDown(status):
            return "Tunnel is either down or about to go down (status: \(status))."
        case .timeout:
            return "Send timeout."
        case let .system(error):
            return "System error: \(error.localizedDescription)"
        }
    }

    var underlyingError: Error? {
        switch self {
        case let .system(error):
            return error
        case .timeout, .tunnelDown:
            return nil
        }
    }
}

struct EmptyTunnelProviderResponseError: LocalizedError {
    var errorDescription: String? {
        return "Unexpected empty (nil) response from the tunnel."
    }
}
