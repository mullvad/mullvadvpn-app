//
//  MullvadAPI.swift
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
    case communication(MullvadAPI.Error)

    /// Mullvad API responded with an error
    case server(MullvadAPI.ResponseError)
}

/// A response received when sending the AppStore receipt to the backend
struct SendAppStoreReceiptResponse: Codable {
    let timeAdded: TimeInterval
    let newExpiry: Date

    /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
    var formattedTimeAdded: String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour]
        formatter.unitsStyle = .full
        formatter.maximumUnitCount = 1

        return formatter.string(from: timeAdded)
    }
}

class MullvadAPI {
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

    /// An error type emitted by `MullvadAPI`
    enum Error: Swift.Error {
        /// A network communication error
        case network(URLError)

        /// An error occured when decoding the JSON response
        case decoding(Swift.Error)

        /// An error occured when encoding the JSON request
        case encoding(Swift.Error)
    }

    typealias ResponseError = JsonRpcResponseError<ResponseCode>
    typealias Response<T: Decodable> = JsonRpcResponse<T, ResponseCode>

    init(session: URLSession = URLSession.shared) {
        self.session = session
    }

    func createAccount() -> AnyPublisher<Response<String>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "create_account", params: [])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func getRelayList() -> AnyPublisher<Response<RelayList>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "relay_list_v3", params: [])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func getAccountExpiry(accountToken: String) -> AnyPublisher<Response<Date>, MullvadAPI.Error> {
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
                    if case .accountDoesNotExist = serverError.code {
                        // Report .invalid account if the server responds with the special code
                        return .invalid
                    } else {
                        // Otherwise report .deferred and pass the server error along
                        return .deferred(.server(serverError))
                    }
                }
            })
            .catch({ (networkError) in
                // Treat all communication errors as .deferred verification
                return Just(.deferred(.communication(networkError)))
            })
            .eraseToAnyPublisher()
    }

    func pushWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<Response<WireguardAssociatedAddresses>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "push_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func replaceWireguardKey(accountToken: String, oldPublicKey: Data, newPublicKey: Data) -> AnyPublisher<Response<WireguardAssociatedAddresses>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "replace_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(oldPublicKey),
            AnyEncodable(newPublicKey)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func checkWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<Response<Bool>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "check_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func removeWireguardKey(accountToken: String, publicKey: Data) -> AnyPublisher<Response<Bool>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "remove_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    func sendAppStoreReceipt(accountToken: String, receiptData: Data) -> AnyPublisher<Response<SendAppStoreReceiptResponse>, MullvadAPI.Error> {
        let request = JsonRpcRequest(method: "apple_payment", params: [
            AnyEncodable(accountToken),
            AnyEncodable(receiptData)
        ])

        return MullvadAPI.makeDataTaskPublisher(request: request)
    }

    private static func makeDataTaskPublisher<T: Decodable>(request: JsonRpcRequest) -> AnyPublisher<Response<T>, MullvadAPI.Error> {
        return Just(request)
            .encode(encoder: makeJSONEncoder())
            .mapError { MullvadAPI.Error.encoding($0) }
            .map { self.makeURLRequest(httpBody: $0) }
            .flatMap {
                URLSession.shared.dataTaskPublisher(for: $0)
                    .mapError { MullvadAPI.Error.network($0) }
                    .flatMap { (data, response) in
                        Just(data)
                            .decode(type: Response<T>.self, decoder: makeJSONDecoder())
                            .mapError { MullvadAPI.Error.decoding($0) }
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
    ResponseCode == MullvadAPI.ResponseCode
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
