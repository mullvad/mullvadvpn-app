//
//  NetworkOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 08/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

extension REST {

    enum RetryAction {
        /// Retry request using next endpoint.
        case useNextEndpoint

        /// Retry request using current endpoint.
        case useCurrentEndpoint

        /// Fail immediately.
        case failImmediately
    }

    class NetworkOperation<Success>: AsyncOperation {
        typealias CompletionHandler = (Result<Success, REST.Error>) -> Void
        typealias Generator = (AnyIPEndpoint, @escaping CompletionHandler) -> Result<URLSessionTask, REST.Error>

        private let networkTaskGenerator: Generator
        private let addressCacheStore: AddressCache.Store
        private var completionHandler: CompletionHandler?
        private var sessionTask: URLSessionTask?

        private let retryStrategy: RetryStrategy
        private var retryTimer: DispatchSourceTimer?
        private var retryCount = 0

        private let logger = Logger(label: "REST.NetworkOperation")
        private let loggerMetadata: Logger.Metadata

        init(name: String, networkTaskGenerator: @escaping Generator, addressCacheStore: AddressCache.Store, retryStrategy: RetryStrategy, completionHandler: @escaping CompletionHandler) {
            self.networkTaskGenerator = networkTaskGenerator
            self.addressCacheStore = addressCacheStore
            self.retryStrategy = retryStrategy
            self.completionHandler = completionHandler

            loggerMetadata = ["requestID": .string(UUID().uuidString), "name": .string(name)]
        }

        override func cancel() {
            DispatchQueue.main.async {
                super.cancel()

                // Cancel pending retry
                self.retryTimer?.cancel()

                // Cancel active network task
                self.sessionTask?.cancel()
            }
        }

        override func main() {
            DispatchQueue.main.async {
                // Finish immediately if operation was cancelled before execution
                guard !self.isCancelled else {
                    self.finish(with: .failure(Self.cancellationError))
                    return
                }

                // Get current endpoint
                self.addressCacheStore.getCurrentEndpoint { endpoint in
                    DispatchQueue.main.async {
                        self.sendRequest(endpoint: endpoint) { [weak self] result in
                            self?.finish(with: result)
                        }
                    }
                }
            }
        }

        private func sendRequest(endpoint: AnyIPEndpoint, completionHandler: @escaping CompletionHandler) {
            // Handle operation cancellation
            guard !isCancelled else {
                completionHandler(.failure(Self.cancellationError))
                return
            }

            // Create network task and execute it
            let taskResult = networkTaskGenerator(endpoint) { [weak self] result in
                DispatchQueue.main.async {
                    self?.handleResponse(endpoint: endpoint, result: result, completionHandler: completionHandler)
                }
            }

            switch taskResult {
            case .success(let dataTask):
                logger.debug("Executing request using \(endpoint)", metadata: loggerMetadata)

                sessionTask = dataTask
                dataTask.resume()

            case .failure(let error):
                logger.error(chainedError: error, message: "Failed to create data task", metadata: loggerMetadata)

                completionHandler(.failure(error))
            }
        }

        private func handleResponse(endpoint: AnyIPEndpoint, result: Result<Success, REST.Error>, completionHandler: @escaping CompletionHandler) {
            guard case .failure(let error) = result else {
                completionHandler(result)
                return
            }

            logger.debug("Failed to perform request to \(endpoint)", metadata: self.loggerMetadata)

            switch Self.evaluateError(error) {
            case .useNextEndpoint:
                // Pick next endpoint in the event of network error
                addressCacheStore.selectNextEndpoint(endpoint) { nextEndpoint in
                    DispatchQueue.main.async {
                        self.retryRequest(endpoint: nextEndpoint, previousResult: result, completionHandler: completionHandler)
                    }
                }

            case .useCurrentEndpoint:
                // Retry request using the same endpoint otherwise
                retryRequest(endpoint: endpoint, previousResult: result, completionHandler: completionHandler)

            case .failImmediately:
                // Fail immediately in case of other errors, like server errors
                completionHandler(result)
            }
        }

        private func retryRequest(endpoint: AnyIPEndpoint, previousResult: Result<Success, REST.Error>, completionHandler: @escaping CompletionHandler) {
            // Handle operation cancellation
            guard !isCancelled else {
                completionHandler(.failure(Self.cancellationError))
                return
            }

            // Increment retry count
            retryCount += 1

            // Check if retry count is not exceeded.
            guard retryCount < retryStrategy.maxRetryCount else {
                logger.debug("Ran out of retry attempts (\(retryStrategy.maxRetryCount))", metadata: loggerMetadata)

                completionHandler(previousResult)
                return
            }

            // Retry immediatly if retry delay is set to .never
            guard retryStrategy.retryDelay != .never else {
                sendRequest(endpoint: endpoint, completionHandler: completionHandler)
                return
            }

            // Create timer to delay retry
            retryTimer = DispatchSource.makeTimerSource(queue: .main)

            retryTimer?.setEventHandler { [weak self] in
                self?.sendRequest(endpoint: endpoint, completionHandler: completionHandler)
            }

            retryTimer?.setCancelHandler {
                completionHandler(.failure(Self.cancellationError))
            }

            retryTimer?.schedule(wallDeadline: .now() + retryStrategy.retryDelay)
            retryTimer?.activate()
        }

        private func finish(with result: Result<Success, REST.Error>) {
            completionHandler?(result)
            completionHandler = nil

            finish()
        }

        private static func evaluateError(_ error: REST.Error) -> RetryAction {
            guard case .network(let networkError) = error else {
                return .failImmediately
            }

            switch networkError.code {
            case .cancelled:
                return .failImmediately

            case .notConnectedToInternet, .internationalRoamingOff, .callIsActive:
                return .useCurrentEndpoint

            default:
                return .useNextEndpoint
            }
        }

        private static var cancellationError: REST.Error {
            return .network(URLError(.cancelled))
        }
    }

}
