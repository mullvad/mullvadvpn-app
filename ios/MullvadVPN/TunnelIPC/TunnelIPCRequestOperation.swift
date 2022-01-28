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
        /// Wait until the tunnel transitioned from reasserting to connected state before sending
        /// the request.
        var waitIfReasserting: Bool

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
        private var timeoutTimer: DispatchSourceTimer?

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

            startTimeoutTimer()

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

        private func startTimeoutTimer() {
            let timer = DispatchSource.makeTimerSource(queue: queue)
            timer.setEventHandler { [weak self] in
                self?.completeOperation(completion: .failure(.send(.timeout)))
            }

            timer.schedule(wallDeadline: .now() + options.timeout)
            timer.activate()

            timeoutTimer = timer
        }

        private func stopTimeoutTimer() {
            timeoutTimer?.cancel()
            timeoutTimer = nil
        }

        private func handleVPNStatus(_ status: NEVPNStatus) {
            guard !isCancelled else {
                return
            }

            switch status {
            case .connected:
                sendRequest()

            case .connecting:
                // Sending IPC message while in connecting state may cause the tunnel process to
                // freeze for no apparent reason.
                break

            case .reasserting:
                if !options.waitIfReasserting {
                    sendRequest()
                }

            case .invalid, .disconnecting, .disconnected:
                completeOperation(completion: .failure(.send(.tunnelDown(status))))

            @unknown default:
                break
            }
        }

        private func sendRequest() {
            let session = connection as! VPNTunnelProviderSessionProtocol

            removeVPNStatusObserver()

            let messageData: Data
            do {
                messageData = try TunnelIPC.Coding.encodeRequest(request)
            } catch {
                completeOperation(completion: .failure(.encoding(error)))
                return
            }

            do {
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
            removeVPNStatusObserver()
            stopTimeoutTimer()

            completionHandler?(completion)
            completionHandler = nil

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
