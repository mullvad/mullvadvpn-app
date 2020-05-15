//
//  MullvadRpc.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import Combine

/// API server URL
private let kMullvadAPIURL = URL(string: "https://api.mullvad.net/rpc/")!

/// Network request timeout in seconds
private let kNetworkTimeout: TimeInterval = 10

/// A response received when sending the AppStore receipt to the backend
struct SendAppStoreReceiptResponse: Codable {
    let timeAdded: TimeInterval
    let newExpiry: Date

    /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
    var formattedTimeAdded: String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour]
        formatter.unitsStyle = .full

        return formatter.string(from: timeAdded)
    }
}

class MullvadRpc {
    private let session: URLSession

    /// A enum mapping the integer codes returned by Mullvad API with the corresponding enum
    /// variants
    private enum RawResponseCode: Int {
        case accountDoesNotExist = -200
        case tooManyWireguardKeys = -703
    }

    /// A enum describing the Mullvad API response code
    enum ResponseCode: RawRepresentable, Codable {
        var rawValue: Int {
            switch self {
            case .accountDoesNotExist:
                return RawResponseCode.accountDoesNotExist.rawValue

            case .tooManyWireguardKeys:
                return RawResponseCode.tooManyWireguardKeys.rawValue

            case .other(let value):
                return value
            }
        }

        init?(rawValue: Int) {
            switch RawResponseCode(rawValue: rawValue) {
            case .accountDoesNotExist:
                self = .accountDoesNotExist
            case .tooManyWireguardKeys:
                self = .tooManyWireguardKeys
            case .none:
                self = ResponseCode.other(rawValue)
            }
        }

        case accountDoesNotExist
        case tooManyWireguardKeys
        case other(Int)
    }

    /// An error type emitted by `MullvadRpc`
    enum Error: Swift.Error {
        /// A network communication error
        case network(URLError)

        /// A server error
        case server(JsonRpcResponseError<ResponseCode>)

        /// An error occured when decoding the JSON response
        case decoding(Swift.Error)

        /// An error occured when encoding the JSON request
        case encoding(Swift.Error)

        var localizedDescription: String {
            switch self {
            case .network(let urlError):
                return "Network error: \(urlError.localizedDescription)"

            case .server(let serverError):
                return "Server error: \(serverError.localizedDescription)"

            case .encoding(let encodingError):
                return "Encoding error: \(encodingError.localizedDescription)"

            case .decoding(let decodingError):
                return "Decoding error: \(decodingError.localizedDescription)"
            }
        }
    }

    init(session: URLSession = URLSession.shared) {
        self.session = session
    }

    func createAccount() -> AnyPublisher<String, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "create_account", params: [])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func getRelayList() -> AnyPublisher<RelayList, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "relay_list_v3", params: [])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func getAccountExpiry(accountToken: String) -> AnyPublisher<Date, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "get_expiry", params: [AnyEncodable(accountToken)])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func pushWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<WireguardAssociatedAddresses, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "push_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func replaceWireguardKey(accountToken: String, oldPublicKey: Data, newPublicKey: Data) -> AnyPublisher<WireguardAssociatedAddresses, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "replace_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(oldPublicKey),
            AnyEncodable(newPublicKey)
        ])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func checkWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<Bool, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "check_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func removeWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<Bool, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "remove_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    func sendAppStoreReceipt(accountToken: String, receiptData: Data) -> AnyPublisher<SendAppStoreReceiptResponse, MullvadRpc.Error> {
        let request = JsonRpcRequest(method: "apple_payment", params: [
            AnyEncodable(accountToken),
            AnyEncodable(receiptData)
        ])

        return MullvadRpc.makeDataTaskPublisher(request: request)
    }

    private static func makeDataTaskPublisher<T: Decodable>(request: JsonRpcRequest) -> AnyPublisher<T, MullvadRpc.Error> {
        return Just(request)
            .encode(encoder: makeJSONEncoder())
            .mapError { MullvadRpc.Error.encoding($0) }
            .map { self.makeURLRequest(httpBody: $0) }
            .flatMap {
                URLSession.shared.dataTaskPublisher(for: $0)
                    .mapError { MullvadRpc.Error.network($0) }
                    .flatMap { (data, httpResponse) in
                        Just(data)
                            .decode(type: JsonRpcResponse<T, ResponseCode>.self, decoder: makeJSONDecoder())
                            .mapError { MullvadRpc.Error.decoding($0) }
                            .flatMap { (serverResponse) in
                                // unwrap JsonRpcResponse.result
                                serverResponse.result
                                    .mapError { MullvadRpc.Error.server($0) }
                                    .publisher
                            }
                }
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


extension JsonRpcResponseError: LocalizedError
    where
    ResponseCode == MullvadRpc.ResponseCode
{
    var errorDescription: String? {
        switch code {
        case .accountDoesNotExist:
            return NSLocalizedString("Invalid account", comment: "")

        case .tooManyWireguardKeys:
            return NSLocalizedString("Too many public WireGuard keys", comment: "")

        case .other:
            return nil
        }
    }

    var recoverySuggestion: String? {
        switch code {
        case .tooManyWireguardKeys:
            return NSLocalizedString("Remove unused WireGuard keys", comment: "")

        default:
            return nil
        }
    }
}
