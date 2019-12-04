//
//  MullvadAPI.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network
import Combine

/// API server URL
private let kMullvadAPIURL = URL(string: "https://api.mullvad.net/rpc/")!

/// Network request timeout in seconds
private let kNetworkTimeout: TimeInterval = 10

/// An error type emitted by `MullvadAPI`
enum MullvadAPIError: Error {
    /// A network communication error
    case network(URLError)

    /// An error occured when decoding the JSON response
    case decoding(Error)

    /// An error occured when encoding the JSON request
    case encoding(Error)
}

/// A type that describes the account verification result
enum AccountVerification {
    /// The app should attempt to verify the account token at some point later because the network
    /// may not be available at this time.
    case deferred(DeferReasonError)

    /// The app successfully verified the account token with the server
    case verified(Date)

    // Invalid token
    case invalid
}

/// An error type that describes why the account verification was deferred
enum DeferReasonError: Error {
    /// Mullvad API communication error
    case communication(MullvadAPIError)

    /// Mullvad API responded with an error
    case server(JsonRpcResponseError)
}

/// The error code returned by the API when it cannot find the given account token
private let kAccountDoesNotExistErrorCode = -200

class MullvadAPI {
    private let session: URLSession

    init(session: URLSession = URLSession.shared) {
        self.session = session
    }

    func getRelayList() -> AnyPublisher<JsonRpcResponse<RelayList>, MullvadAPIError> {
        let request = JsonRpcRequest(method: "relay_list_v3", params: [])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func getAccountExpiry(accountToken: String) -> AnyPublisher<JsonRpcResponse<Date>, MullvadAPIError> {
        let request = JsonRpcRequest(method: "get_expiry", params: [AnyEncodable(accountToken)])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func verifyAccount(accountToken: String) -> AnyPublisher<AccountVerification, Never> {
        return getAccountExpiry(accountToken: accountToken)
            .map({ (response) -> AccountVerification in
                switch response.result {
                case .success(let expiry):
                    // Report .verified when expiry is successfully received
                    return .verified(expiry)

                case .failure(let serverError):
                    if serverError.code == kAccountDoesNotExistErrorCode {
                        // Report .invalid account if the server responds with the special code
                        return .invalid
                    } else {
                        // Otherwise report .deferred and pass the server error along
                        return .deferred(.server(serverError))
                    }
                }
            })
            .catch({ (networkError) in
                // Treat all network errors as .deferred verification
                return Just(.deferred(.communication(networkError)))
            })
            .eraseToAnyPublisher()
    }

    func pushWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<JsonRpcResponse<WireguardAssociatedAddresses>, MullvadAPIError> {
        let request = JsonRpcRequest(method: "push_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func checkWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<JsonRpcResponse<WireguardAssociatedAddresses>, MullvadAPIError> {
        let request = JsonRpcRequest(method: "check_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    private static func makeDataTaskPublisher<T: Decodable>(request: JsonRpcRequest) -> AnyPublisher<JsonRpcResponse<T>, MullvadAPIError> {
        return Just(request)
            .encode(encoder: makeJSONEncoder())
            .mapError { MullvadAPIError.encoding($0) }
            .map { self.makeURLRequest(httpBody: $0) }
            .flatMap {
                URLSession.shared.dataTaskPublisher(for: $0)
                    .mapError { MullvadAPIError.network($0) }
                    .map { $0.data }
                    .decode(type: JsonRpcResponse<T>.self, decoder: makeJSONDecoder())
                    .mapError { MullvadAPIError.decoding($0) }
        }.eraseToAnyPublisher()
    }

    private static func makeURLRequest(httpBody: Data) -> URLRequest {
        var request = URLRequest(url: kMullvadAPIURL, cachePolicy: .useProtocolCachePolicy, timeoutInterval: kNetworkTimeout)
        request.addValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpMethod = "POST"
        request.httpBody = httpBody

        return request
    }

    private static func makeJSONEncoder() -> JSONEncoder {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dateEncodingStrategy = .iso8601
        encoder.dataEncodingStrategy = .base64
        return encoder
    }

    private static func makeJSONDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        decoder.dateDecodingStrategy = .iso8601
        decoder.dataDecodingStrategy = .base64
        return decoder
    }
}
