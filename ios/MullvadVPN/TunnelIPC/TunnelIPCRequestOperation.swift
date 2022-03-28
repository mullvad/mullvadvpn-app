//
//  TunnelIPCRequestOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

extension TunnelIPC {

    struct RequestOptions {
        /// Delay for sending IPC requests to the tunnel when in connecting state.
        /// Used to workaround a bug when talking to the tunnel too early may cause it to freeze.
        static let connectingStateWaitDelay: TimeInterval = 5

        /// Timeout interval in seconds.
        var timeout: TimeInterval = 5
    }

    final class RequestOperation<Output>: ResultOperation<Output, TunnelIPC.Error> {
        typealias DecoderHandler = (Data?) -> Result<Output, TunnelIPC.Error>

        private let queue: DispatchQueue

        private let tunnel: Tunnel
        private let request: TunnelIPC.Request
        private let options: RequestOptions

        private let decoderHandler: DecoderHandler

        private var statusObserver: Tunnel.StatusBlockObserver?
        private var timeoutWork: DispatchWorkItem?
        private var waitForConnectingStateWork: DispatchWorkItem?

        private var requestSent = false

        init(queue: DispatchQueue,
             tunnel: Tunnel,
             request: TunnelIPC.Request,
             options: TunnelIPC.RequestOptions,
             decoderHandler: @escaping DecoderHandler,
             completionHandler: @escaping CompletionHandler)
        {
            self.queue = queue

            self.tunnel = tunnel
            self.request = request
            self.options = options

            self.decoderHandler = decoderHandler

            super.init(completionQueue: queue, completionHandler: completionHandler)
        }

        override func main() {
            queue.async {
                guard !self.isCancelled else {
                    self.completeOperation(completion: .cancelled)
                    return
                }

                self.setTimeoutTimer(connectingStateWaitDelay: 0)

                self.statusObserver = self.tunnel.addBlockObserver(queue: self.queue) { [weak self] tunnel, status in
                    self?.handleVPNStatus(status)
                }

                self.handleVPNStatus(self.tunnel.status)
            }
        }

        override func cancel() {
            super.cancel()

            queue.async {
                if self.isExecuting {
                    self.completeOperation(completion: .cancelled)
                }
            }
        }

        private func removeVPNStatusObserver() {
            statusObserver?.invalidate()
            statusObserver = nil
        }

        private func setTimeoutTimer(connectingStateWaitDelay: TimeInterval) {
            let workItem = DispatchWorkItem { [weak self] in
                self?.completeOperation(completion: .failure(.send(.timeout)))
            }

            // Cancel pending timeout work.
            timeoutWork?.cancel()

            // Assign new timeout work.
            timeoutWork = workItem

            // Schedule timeout work.
            let deadline: DispatchWallTime = .now() + options.timeout + connectingStateWaitDelay

            queue.asyncAfter(wallDeadline: deadline, execute: workItem)
        }

        private func handleVPNStatus(_ status: NEVPNStatus) {
            guard !isCancelled && !requestSent else {
                return
            }

            switch status {
            case .connected:
                sendRequest()

            case .connecting:
                waitForConnectingState { [weak self] in
                    self?.sendRequest()
                }

            case .reasserting:
                sendRequest()

            case .invalid, .disconnecting, .disconnected:
                completeOperation(completion: .failure(.send(.tunnelDown(status))))

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
            guard timeElapsed < RequestOptions.connectingStateWaitDelay else {
                block()
                return
            }

            let waitDelay = RequestOptions.connectingStateWaitDelay - timeElapsed
            let workItem = DispatchWorkItem(block: block)

            // Assign new work.
            waitForConnectingStateWork = workItem

            // Reschedule the timeout work.
            setTimeoutTimer(connectingStateWaitDelay: waitDelay)

            // Schedule delayed work.
            let deadline: DispatchWallTime = .now() + waitDelay

            queue.asyncAfter(wallDeadline: deadline, execute: workItem)
        }

        private func sendRequest() {
            // Mark request sent.
            requestSent = true

            // Release status observer.
            removeVPNStatusObserver()

            // Cancel pending delayed work.
            waitForConnectingStateWork?.cancel()

            // Encode request.
            let messageData: Data
            do {
                messageData = try TunnelIPC.Coding.encodeRequest(request)
            } catch {
                completeOperation(completion: .failure(.encoding(error)))
                return
            }

            // Send IPC message.
            do {
                try tunnel.sendProviderMessage(messageData) { [weak self] responseData in
                    guard let self = self else { return }

                    self.queue.async {
                        let decodingResult = self.decoderHandler(responseData)

                        self.completeOperation(completion: OperationCompletion(result: decodingResult))
                    }
                }
            } catch {
                completeOperation(completion: .failure(.send(.system(error))))
            }
        }

        private func completeOperation(completion: OperationCompletion<Output, TunnelIPC.Error>) {
            // Release status observer.
            removeVPNStatusObserver()

            // Cancel pending work.
            timeoutWork?.cancel()
            waitForConnectingStateWork?.cancel()

            // Finish operation.
            finish(completion: completion)
        }
    }
}

extension TunnelIPC.RequestOperation where Output: Codable {
    convenience init(
        queue: DispatchQueue,
        tunnel: Tunnel,
        request: TunnelIPC.Request,
        options: TunnelIPC.RequestOptions,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.init(
            queue: queue,
            tunnel: tunnel,
            request: request,
            options: options,
            decoderHandler: { data in
                guard let data = data else {
                    return .failure(.nilResponse)
                }

                let result = Result { try TunnelIPC.Coding.decodeResponse(Output.self, from: data) }

                return result.mapError { .decoding($0) }
            },
            completionHandler: completionHandler
        )
    }
}

extension TunnelIPC.RequestOperation where Output == Void {
    convenience init(
        queue: DispatchQueue,
        tunnel: Tunnel,
        request: TunnelIPC.Request,
        options: TunnelIPC.RequestOptions,
        completionHandler: @escaping CompletionHandler
    ) {
        self.init(
            queue: queue,
            tunnel: tunnel,
            request: request,
            options: options,
            decoderHandler: { _ in .success(()) },
            completionHandler: completionHandler
        )
    }
}
