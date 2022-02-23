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

    final class RequestOperation<Output>: AsyncOperation {
        typealias DecoderHandler = (Data?) -> Result<Output, TunnelIPC.Error>
        typealias CompletionHandler = (OperationCompletion<Output, TunnelIPC.Error>) -> Void

        private let queue: DispatchQueue
        private let notificationQueue: OperationQueue

        private let connection: VPNConnectionProtocol
        private let request: TunnelIPC.Request
        private let options: RequestOptions

        private let decoderHandler: DecoderHandler
        private var completionHandler: CompletionHandler?

        private var statusObserver: NSObjectProtocol?
        private var timeoutWork: DispatchWorkItem?
        private var waitForConnectingStateWork: DispatchWorkItem?

        init(queue: DispatchQueue,
             connection: VPNConnectionProtocol,
             request: TunnelIPC.Request,
             options: TunnelIPC.RequestOptions,
             decoderHandler: @escaping DecoderHandler,
             completionHandler: @escaping CompletionHandler)
        {
            self.queue = queue
            self.notificationQueue = OperationQueue()
            self.notificationQueue.underlyingQueue = queue

            self.connection = connection
            self.request = request
            self.options = options

            self.decoderHandler = decoderHandler
            self.completionHandler = completionHandler
        }

        override func main() {
            queue.async {
                self.execute()
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

        private func execute() {
            guard !isCancelled else {
                completeOperation(completion: .cancelled)
                return
            }

            setTimeoutTimer(isConnectingState: false)

            statusObserver = NotificationCenter.default.addObserver(
                forName: .NEVPNStatusDidChange,
                object: connection,
                queue: notificationQueue) { [weak self] notification in
                    guard let self = self else { return }
                    guard let connection = notification.object as? VPNConnectionProtocol else { return }

                    self.handleVPNStatus(connection.status)
                }

            handleVPNStatus(connection.status)
        }

        private func removeVPNStatusObserver() {
            if let statusObserver = statusObserver {
                NotificationCenter.default.removeObserver(statusObserver)
                self.statusObserver = nil
            }
        }

        private func setTimeoutTimer(isConnectingState: Bool) {
            let workItem = DispatchWorkItem { [weak self] in
                self?.completeOperation(completion: .failure(.send(.timeout)))
            }

            // Cancel pending timeout work.
            timeoutWork?.cancel()

            // Assign new timeout work.
            timeoutWork = workItem

            // Compute additional delay associated with connecting state.
            let connectingStateWaitDelay = isConnectingState ? RequestOptions.connectingStateWaitDelay : 0

            // Schedule timeout work.
            let deadline: DispatchWallTime = .now() + options.timeout + connectingStateWaitDelay

            queue.asyncAfter(wallDeadline: deadline, execute: workItem)
        }

        private func handleVPNStatus(_ status: NEVPNStatus) {
            guard !isCancelled else {
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
            let workItem = DispatchWorkItem(block: block)

            // Cancel pending work.
            waitForConnectingStateWork?.cancel()

            // Assign new work.
            waitForConnectingStateWork = workItem

            // Reschedule the timeout work.
            setTimeoutTimer(isConnectingState: true)

            // Schedule delayed work.
            let deadline: DispatchWallTime = .now() + RequestOptions.connectingStateWaitDelay

            queue.asyncAfter(wallDeadline: deadline, execute: workItem)
        }

        private func sendRequest() {
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
                let session = connection as! VPNTunnelProviderSessionProtocol
                try session.sendProviderMessage(messageData) { [weak self] responseData in
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

            // Call completion handler.
            completionHandler?(completion)
            completionHandler = nil

            // Finish operation.
            finish()
        }
    }
}

extension TunnelIPC.RequestOperation where Output: Codable {
    convenience init(
        queue: DispatchQueue,
        connection: VPNConnectionProtocol,
        request: TunnelIPC.Request,
        options: TunnelIPC.RequestOptions,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.init(
            queue: queue,
            connection: connection,
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
        connection: VPNConnectionProtocol,
        request: TunnelIPC.Request,
        options: TunnelIPC.RequestOptions,
        completionHandler: @escaping CompletionHandler
    ) {
        self.init(
            queue: queue,
            connection: connection,
            request: request,
            options: options,
            decoderHandler: { _ in .success(()) },
            completionHandler: completionHandler
        )
    }
}
