//
//  MullvadApiNetworkOperation.swift
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

private enum MullvadApiTransportError: Error {
    case connectionFailed(description: String?)
}

extension REST {
    class MullvadApiNetworkOperation<Success: Sendable>: ResultOperation<Success>, @unchecked Sendable {
        private let logger: Logger

        private let request: APIRequest
        private let transportProvider: APITransportProviderProtocol
        private var responseDecoder: JSONDecoder
        private let responseHandler: any RESTRustResponseHandler<Success>
        private var networkTask: Cancellable?

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            request: APIRequest,
            transportProvider: APITransportProviderProtocol,
            responseDecoder: JSONDecoder,
            responseHandler: some RESTRustResponseHandler<Success>,
            completionHandler: CompletionHandler? = nil
        ) {
            self.request = request
            self.transportProvider = transportProvider
            self.responseDecoder = responseDecoder
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
            networkTask?.cancel()
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

            let transport = transportProvider.makeTransport()
            do {
                networkTask = try transport?.sendRequest(request) { [weak self] response in
                    guard let self else { return }

                    if let apiError = response.error {
                        finish(result: .failure(restError(apiError: apiError)))
                        return
                    }

                    let decodedResponse = responseHandler.handleResponse(response)

                    switch decodedResponse {
                    case let .success(value):
                        finish(result: .success(value))
                    case let .decoding(block):
                        do {
                            finish(result: .success(try block()))
                        } catch {
                            finish(result: .failure(REST.Error.unhandledResponse(0, nil)))
                        }
                    case let .unhandledResponse(error):
                        finish(result: .failure(REST.Error.unhandledResponse(0, error)))
                    }
                }
            } catch {
                finish(result: .failure(error))
            }
        }

        private func restError(apiError: APIError) -> Error {
            guard let serverResponseCode = apiError.serverResponseCode else {
                return .transport(MullvadApiTransportError.connectionFailed(description: apiError.errorDescription))
            }

            let response = REST.ServerErrorResponse(
                code: REST.ServerResponseCode(rawValue: serverResponseCode),
                detail: apiError.errorDescription
            )
            return .unhandledResponse(apiError.statusCode, response)
        }
    }
}
