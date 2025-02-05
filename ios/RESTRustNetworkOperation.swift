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
    typealias RustRequestFactory = ((((MullvadApiResponse) throws -> Void)?) -> AnyCancellable)

    class RustNetworkOperation<Success: Sendable>: ResultOperation<Success>, @unchecked Sendable {
        private let logger: Logger

        private let requestFactory: RustRequestFactory
        private var responseDecoder: JSONDecoder
        private let responseHandler: any RESTRustResponseHandler<Success>
        private var networkTask: Cancellable?

        private let retryStrategy: RetryStrategy
        private var retryDelayIterator: AnyIterator<Duration>
        private var retryTimer: DispatchSourceTimer?
        private var retryCount = 0

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            retryStrategy: RetryStrategy,
            requestFactory: @escaping RustRequestFactory,
            responseDecoder: JSONDecoder,
            responseHandler: some RESTRustResponseHandler<Success>,
            completionHandler: CompletionHandler? = nil
        ) {
            self.retryStrategy = retryStrategy
            retryDelayIterator = retryStrategy.makeDelayIterator()
            self.responseDecoder = responseDecoder
            self.requestFactory = requestFactory
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

            networkTask = requestFactory({ [weak self] response in
                guard let self else { return }

                if let error = try response.restError(decoder: responseDecoder) {
                    finish(result: .failure(error))
                    return
                }

                // TODO: invoke the retry strategy if the request failed with a transport error, generally the response should contain enough information to make such a judgement call.

                let decodedResponse = responseHandler.handleResponse(response)

                switch decodedResponse {
                case .success(let value):
                    finish(result: .success(value))
                case .decoding(let block):
                    finish(result: .success(try block()))
                case .unhandledResponse(let error):
                    finish(result: .failure(REST.Error.unhandledResponse(Int(response.statusCode), error)))
                }
            })
        }
    }
}

extension MullvadApiResponse {
    // TODO: Construct all the real error types from Rust side.
    public func restError(decoder: JSONDecoder) throws -> REST.Error? {
        guard !success else {
            return nil
        }

        guard let body else {
            return .transport(RustTransportError.connectionFailed(description: errorDescription))
        }

        do {
            let response = try decoder.decode(REST.ServerErrorResponse.self, from: body)
            return .unhandledResponse(Int(statusCode), response)
        } catch {
            return .transport(RustTransportError.connectionFailed(description: errorDescription))
        }
    }
}

// TODO: Add all the things. Or remove and use a proper type.
enum RustTransportError: Error {
    case connectionFailed(description: String?)
}
