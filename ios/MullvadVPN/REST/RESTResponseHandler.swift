//
//  RESTResponseHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 25/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol RESTResponseHandler {
    associatedtype Success

    func handleURLResponse(_ response: HTTPURLResponse, data: Data) -> Result<Success, REST.Error>
}

extension REST {
    final class AnyResponseHandler<Success>: RESTResponseHandler {
        typealias HandlerBlock = (HTTPURLResponse, Data) -> Result<Success, REST.Error>

        private let handlerBlock: HandlerBlock

        init(_ block: @escaping HandlerBlock) {
            handlerBlock = block
        }

        func handleURLResponse(_ response: HTTPURLResponse, data: Data) -> Result<Success, REST.Error> {
            return handlerBlock(response, data)
        }
    }

    /// Returns default response handler that parses JSON response into the
    /// given `Decodable` type when it encounters HTTP `2xx` code, otherwise
    /// attempts to decode the server error.
    static func defaultResponseHandler<T: Decodable>(decoding type: T.Type, with decoder: REST.ResponseDecoder) -> AnyResponseHandler<T> {
        return AnyResponseHandler { response, data in
            if HTTPStatus.isSuccess(response.statusCode) {
                return decoder.decodeSuccessResponse(type, from: data)
            } else {
                return decoder.decodeErrorResponseAndMapToServerError(from: data)
            }
        }
    }
}
