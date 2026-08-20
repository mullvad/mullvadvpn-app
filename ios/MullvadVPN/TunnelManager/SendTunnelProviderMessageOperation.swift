//
//  SendTunnelProviderMessageOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import NetworkExtension
import Operations
import PacketTunnelCore

final class SendTunnelProviderMessageOperation<Output: Sendable>: ResultOperation<Output>, @unchecked Sendable {
    typealias DecoderHandler = (Data?) throws -> Output

    private let backgroundTaskProvider: BackgroundTaskProviding
    private let tunnel: any TunnelProtocol
    private let message: TunnelProviderMessage
    private let timeout: Duration

    private let decoderHandler: DecoderHandler

    private var statusObserver: TunnelStatusBlockObserver?
    private var waitForConnectingStateTask: Task<Void, Never>?

    private var messageSent = false

    /// Delay for sending tunnel provider messages to the tunnel when in connecting state.
    /// Used to workaround a bug when talking to the tunnel too early during startup may cause it
    /// to freeze.
    private let connectingStateWaitDelay: Duration = .seconds(5)

    /// Default timeout in seconds.
    private let defaultTimeout: Duration = .seconds(5)

    init(
        dispatchQueue: DispatchQueue,
        backgroundTaskProvider: BackgroundTaskProviding,
        tunnel: any TunnelProtocol,
        message: TunnelProviderMessage,
        timeout: Duration? = nil,
        decoderHandler: @escaping DecoderHandler,
        completionHandler: CompletionHandler?
    ) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.tunnel = tunnel
        self.message = message
        self.timeout = timeout ?? defaultTimeout

        self.decoderHandler = decoderHandler

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )

        addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Send tunnel provider message: \(message)",
                cancelUponExpiration: true
            )
        )
    }

    override func main() {
        statusObserver = tunnel.addBlockObserver(queue: dispatchQueue) { [weak self] _, status in
            self?.handleVPNStatus(status)
        }

        handleVPNStatus(tunnel.status)
    }

    override func operationDidCancel() {
        finish(result: .failure(OperationError.cancelled))
    }

    override func finish(result: Result<Output, Error>) {
        // Release status observer.
        removeVPNStatusObserver()

        // Cancel pending work.
        waitForConnectingStateTask?.cancel()

        // Finish operation.
        super.finish(result: result)
    }

    private func removeVPNStatusObserver() {
        statusObserver?.invalidate()
        statusObserver = nil
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
            finish(result: .failure(SendTunnelProviderMessageError.tunnelDown(status)))

        @unknown default:
            break
        }
    }

    private func waitForConnectingState(block: @escaping @Sendable () -> Void) {
        Task { [weak self, tunnel] in
            guard let self else { return }

            // Compute amount of time elapsed since the tunnel was launched.
            let timeElapsed: TimeInterval
            if let startDate = await tunnel.startDate {
                timeElapsed = Date().timeIntervalSince(startDate)
            } else {
                timeElapsed = 0
            }

            // Cancel pending work.
            waitForConnectingStateTask?.cancel()
            waitForConnectingStateTask = nil

            // Execute right away if enough time passed since the tunnel was launched.
            guard timeElapsed < connectingStateWaitDelay else {
                block()
                return
            }

            let waitDelay = connectingStateWaitDelay - timeElapsed

            let task = Task {
                try? await Task.sleep(nanoseconds: UInt64(waitDelay.milliseconds * 1_000_000))

                guard !Task.isCancelled else {
                    return
                }

                block()
            }

            waitForConnectingStateTask = task
        }
    }

    private func sendMessage() {
        // Mark message sent.
        messageSent = true

        // Release status observer.
        removeVPNStatusObserver()

        // Cancel pending delayed work.
        waitForConnectingStateTask?.cancel()

        // Encode message.
        let messageData: Data
        do {
            messageData = try message.encode()
        } catch {
            finish(result: .failure(error))
            return
        }

        guard backgroundTaskProvider.backgroundTimeRemaining > timeout else {
            finish(result: .failure(SendTunnelProviderMessageError.notEnoughBackgroundTime))
            return
        }

        // Send IPC message.
        do {
            try tunnel.sendProviderMessage(messageData) { [weak self] responseData in
                guard let self else { return }

                dispatchQueue.async {
                    let decodingResult = Result { try self.decoderHandler(responseData) }

                    self.finish(result: decodingResult)
                }
            }
        } catch {
            finish(result: .failure(SendTunnelProviderMessageError.system(error)))
        }
    }
}

enum SendTunnelProviderMessageError: LocalizedError, WrappingError {
    /// Tunnel process is either down or about to go down.
    case tunnelDown(NEVPNStatus)

    /// Timeout.
    case timeout

    /// System error.
    case system(Error)

    /// Not enough background time to accommodate the operation.
    case notEnoughBackgroundTime

    var errorDescription: String? {
        switch self {
        case let .tunnelDown(status):
            return "Tunnel is either down or about to go down (status: \(status))."
        case .timeout:
            return "Send timeout."
        case .notEnoughBackgroundTime:
            return "Not enough background time to accommodate the operation."
        case let .system(error):
            return "System error: \(error.localizedDescription)"
        }
    }

    var underlyingError: Error? {
        switch self {
        case let .system(error):
            return error
        case .timeout, .tunnelDown, .notEnoughBackgroundTime:
            return nil
        }
    }
}

struct EmptyTunnelProviderResponseError: LocalizedError {
    var errorDescription: String? {
        "Unexpected empty (nil) response from the tunnel."
    }
}
