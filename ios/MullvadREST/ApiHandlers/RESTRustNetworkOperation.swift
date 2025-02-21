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

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            requestHandler: @escaping MullvadApiRequestHandler,
            responseDecoder: JSONDecoder,
            responseHandler: some RESTRustResponseHandler<Success>,
            completionHandler: CompletionHandler? = nil
        ) {
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
            do {
                networkTask = try requestHandler { [weak self] response in
                    guard let self else { return }

                    if let error = response.restError() {
                        finish(result: .failure(error))
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
            } catch {
                finish(result: .failure(REST.Error.createURLRequest(error)))
            }
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
