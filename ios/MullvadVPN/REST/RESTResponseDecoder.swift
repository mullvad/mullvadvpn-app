//
//  RESTResponseDecoder.swift
//  MullvadVPN
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    struct ResponseDecoder {
        let decoder: JSONDecoder

        init(decoder: JSONDecoder) {
            self.decoder = decoder
        }

        // Parse JSON response into the given `Decodable` type.
        func decodeSuccessResponse<T: Decodable>(_ type: T.Type, from data: Data) -> Result<T, REST.Error> {
            return Result { try decoder.decode(type, from: data) }
                .mapError { error in
                    return .decodeSuccessResponse(error)
                }
        }

        /// Parse server error response from JSON.
        func decodeErrorResponse(from data: Data) -> Result<REST.ServerErrorResponse, REST.Error> {
            return Result { () -> REST.ServerErrorResponse in
                return try decoder.decode(REST.ServerErrorResponse.self, from: data)
            }
            .mapError { error in
                return .decodeErrorResponse(error)
            }
        }

        /// Parse server error response from JSON and map it to `RESTError.server` error kind.
        func decodeErrorResponseAndMapToServerError<T>(from data: Data) -> Result<T, REST.Error> {
            return decodeErrorResponse(from: data)
                .flatMap { serverError in
                    return .failure(.server(serverError))
                }
        }
    }

}
