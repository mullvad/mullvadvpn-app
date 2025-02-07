//
//  RESTRustNetworkOperation.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2025-01-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadRustRuntime
import MullvadTypes
import Operations

extension REST {
    class MullvadApiNetworkOperation<Success: Sendable>: ResultOperation<Success>, @unchecked Sendable {
        private let logger: Logger

        private let requestHandler: MullvadApiRequestHandler
        private var responseDecoder: JSONDecoder
        private let responseHandler: any RESTRustResponseHandler<Success>
        private var networkTask: MullvadApiCancellable?

        private let retryStrategy: RetryStrategy
        private var retryDelayIterator: AnyIterator<Duration>
        private var retryTimer: DispatchSourceTimer?
        private var retryCount = 0

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            retryStrategy: RetryStrategy,
            requestHandler: @escaping MullvadApiRequestHandler,
            responseDecoder: JSONDecoder,
            responseHandler: some RESTRustResponseHandler<Success>,
            completionHandler: CompletionHandler? = nil
        ) {
            self.retryStrategy = retryStrategy
            retryDelayIterator = retryStrategy.makeDelayIterator()
            self.responseDecoder = responseDecoder
            self.requestHandler = requestHandler
            self.responseHandler = responseHandler

            var logger = Logger(label: "REST.RustNetworkOperation")
            logger[metadataKey: "name"] = .string(name)
            self.logger = logger

            super.init(
                dispatchQueue: dispatchQueue,
                completionQueue: .main,
                completionHandler: completionHandler
            )
        }

        override public func operationDidCancel() {
            retryTimer?.cancel()
            networkTask?.cancel()

            retryTimer = nil
            networkTask = nil
        }

        override public func main() {
            startRequest()
        }

        func startRequest() {
            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(result: .failure(OperationError.cancelled))
                return
            }

            networkTask = requestHandler { [weak self] response in
                guard let self else { return }

                if let error = response.restError() {
                    if response.shouldRetry {
                        retryRequest(with: error)
                    } else {
                        finish(result: .failure(error))
                    }

                    return
                }

                let decodedResponse = responseHandler.handleResponse(response)

                switch decodedResponse {
                case let .success(value):
                    finish(result: .success(value))
                case let .decoding(block):
                    finish(result: .success(try block()))
                case let .unhandledResponse(error):
                    finish(result: .failure(REST.Error.unhandledResponse(Int(response.statusCode), error)))
                }
            }
        }

        private func retryRequest(with error: REST.Error) {
            // Check if retry count is not exceeded.
            guard retryCount < retryStrategy.maxRetryCount else {
                if retryStrategy.maxRetryCount > 0 {
                    logger.debug("Ran out of retry attempts (\(retryStrategy.maxRetryCount))")
                }
                finish(result: .failure(error))
                return
            }

            // Increment retry count.
            retryCount += 1

            // Retry immediately if retry delay is set to never.
            guard retryStrategy.delay != .never else {
                startRequest()
                return
            }

            guard let waitDelay = retryDelayIterator.next() else {
                logger.debug("Retry delay iterator failed to produce next value.")

                finish(result: .failure(error))
                return
            }

            logger.debug("Retry in \(waitDelay.logFormat()).")

            // Create timer to delay retry.
            let timer = DispatchSource.makeTimerSource(queue: dispatchQueue)

            timer.setEventHandler { [weak self] in
                self?.startRequest()
            }

            timer.setCancelHandler { [weak self] in
                self?.finish(result: .failure(OperationError.cancelled))
            }

            timer.schedule(wallDeadline: .now() + waitDelay.timeInterval)
            timer.activate()

            retryTimer = timer
        }
    }
}

extension MullvadApiResponse {
    public func restError() -> REST.Error? {
        guard !success else {
            return nil
        }

        guard let serverResponseCode else {
            return .transport(MullvadApiTransportError.connectionFailed(description: errorDescription))
        }

        let response = REST.ServerErrorResponse(
            code: REST.ServerResponseCode(rawValue: serverResponseCode),
            detail: errorDescription
        )
        return .unhandledResponse(Int(statusCode), response)
    }
}

enum MullvadApiTransportError: Error {
    case connectionFailed(description: String?)
}
