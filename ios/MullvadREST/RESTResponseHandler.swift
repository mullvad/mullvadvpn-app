//
//  RESTResponseHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 25/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol RESTResponseHandler {
    associatedtype Success

    func handleURLResponse(_ response: HTTPURLResponse, data: Data) -> REST
        .ResponseHandlerResult<Success>
}

public extension REST {
    /// Responser handler result type.
    enum ResponseHandlerResult<Success> {
        /// Response handler succeeded and produced a value.
        case success(Success)

        /// Response handler succeeded and returned a block that decodes the value.
        case decoding(_ decoderBlock: () throws -> Success)

        /// Response handler received the response that it cannot handle.
        /// Server error response is attached when available.
        case unhandledResponse(ServerErrorResponse?)
    }

    final class AnyResponseHandler<Success>: RESTResponseHandler {
        public typealias HandlerBlock = (HTTPURLResponse, Data) -> REST
            .ResponseHandlerResult<Success>

        private let handlerBlock: HandlerBlock

        public init(_ block: @escaping HandlerBlock) {
            handlerBlock = block
        }

        public func handleURLResponse(_ response: HTTPURLResponse, data: Data) -> REST
            .ResponseHandlerResult<Success>
        {
            return handlerBlock(response, data)
        }
    }

    /// Returns default response handler that parses JSON response into the
    /// given `Decodable` type when it encounters HTTP `2xx` code, otherwise
    /// attempts to decode the server error.
    static func defaultResponseHandler<T: Decodable>(
        decoding type: T.Type,
        with decoder: JSONDecoder
    ) -> AnyResponseHandler<T> {
        return AnyResponseHandler { response, data in
            if HTTPStatus.isSuccess(response.statusCode) {
                return .decoding {
                    try decoder.decode(type, from: data)
                }
            } else {
                return .unhandledResponse(
                    try? decoder.decode(
                        ServerErrorResponse.self,
                        from: data
                    )
                )
            }
        }
    }
}
