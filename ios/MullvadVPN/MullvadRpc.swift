//
//  MullvadRpc.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

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
    enum Error: ChainedError {
        /// A network communication error
        case network(URLError)

        /// A server error
        case server(JsonRpcResponseError<ResponseCode>)

        /// An error occured when decoding the JSON response
        case decoding(Swift.Error)

        /// An error occured when encoding the JSON request
        case encoding(Swift.Error)

        var errorDescription: String? {
            switch self {
            case .network:
                return "Network error"

            case .server:
                return "Server error"

            case .encoding:
                return "Encoding error"

            case .decoding:
                return "Decoding error"
            }
        }
    }

    /// Returns an instance of `MullvadRpc` configured with ephemeral `URLSession` configuration
    class func withEphemeralURLSession() -> MullvadRpc {
        return MullvadRpc(session: URLSession(configuration: .ephemeral))
    }

    class func makeJSONEncoder() -> JSONEncoder {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dateEncodingStrategy = .iso8601
        encoder.dataEncodingStrategy = .base64
        return encoder
    }

    class func makeJSONDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        decoder.dateDecodingStrategy = .iso8601
        decoder.dataDecodingStrategy = .base64
        return decoder
    }

    init(session: URLSession) {
        self.session = session
    }

    func createAccount() -> MullvadRpc.Request<String> {
        let request = JsonRpcRequest(method: "create_account", params: [])

        return MullvadRpc.Request(session: session, request: request)
    }

    func getRelayList() -> MullvadRpc.Request<RelayList> {
        let request = JsonRpcRequest(method: "relay_list_v3", params: [])

        return MullvadRpc.Request(session: session, request: request)
    }

    func getAccountExpiry(accountToken: String) -> MullvadRpc.Request<Date> {
        let request = JsonRpcRequest(method: "get_expiry", params: [AnyEncodable(accountToken)])

        return MullvadRpc.Request(session: session, request: request)
    }

    func getAccountExpiry(request: MullvadRpc.Request<Date>? = nil) -> MullvadRpc.Operation<Date> {
        return MullvadRpc.Operation(request: request)
    }

    func pushWireguardKey(accountToken: String, publicKey: Data) -> MullvadRpc.Request<WireguardAssociatedAddresses> {
        let request = JsonRpcRequest(method: "push_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadRpc.Request(session: session, request: request)
    }

    func replaceWireguardKey(accountToken: String, oldPublicKey: Data, newPublicKey: Data) -> MullvadRpc.Request<WireguardAssociatedAddresses> {
        let request = JsonRpcRequest(method: "replace_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(oldPublicKey),
            AnyEncodable(newPublicKey)
        ])

        return MullvadRpc.Request(session: session, request: request)
    }

    func checkWireguardKey(accountToken: String, publicKey: Data) -> MullvadRpc.Request<Bool> {
        let request = JsonRpcRequest(method: "check_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadRpc.Request(session: session, request: request)
    }

    func checkWireguardKey(request: MullvadRpc.Request<Bool>? = nil) -> MullvadRpc.Operation<Bool> {
        return MullvadRpc.Operation(request: request)
    }

    func removeWireguardKey(accountToken: String, publicKey: Data) -> MullvadRpc.Request<Bool> {
        let request = JsonRpcRequest(method: "remove_wg_key", params: [
            AnyEncodable(accountToken),
            AnyEncodable(publicKey)
        ])

        return MullvadRpc.Request(session: session, request: request)
    }

    func sendAppStoreReceipt(accountToken: String, receiptData: Data) -> MullvadRpc.Request<SendAppStoreReceiptResponse> {
        let request = JsonRpcRequest(method: "apple_payment", params: [
            AnyEncodable(accountToken),
            AnyEncodable(receiptData)
        ])

        return MullvadRpc.Request(session: session, request: request)
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


extension MullvadRpc {

    class Request<Response: Decodable> {
        typealias RequestCompletionHandler = (Result<Response, MullvadRpc.Error>) -> Void

        private let session: URLSession
        private let request: JsonRpcRequest

        private let lock = NSLock()
        private var urlSessionTask: URLSessionTask?

        fileprivate init(session: URLSession, request: JsonRpcRequest) {
            self.session = session
            self.request = request
        }

        func start(completionHandler: @escaping RequestCompletionHandler) {
            lock.withCriticalBlock {
                assert(self.urlSessionTask == nil)

                switch makeURLRequest() {
                case .success(let urlRequest):
                    let task = session.dataTask(with: urlRequest) { (responseData, urlResponse, error) in
                        switch (responseData, error) {
                        case (.some(let data), .none):
                            completionHandler(Self.decodeResponse(data))

                        case (.none, .some(let urlError as URLError)):
                            completionHandler(.failure(.network(urlError)))

                        default:
                            fatalError()
                        }
                    }
                    self.urlSessionTask = task
                    task.resume()

                case .failure(let error):
                    completionHandler(.failure(error))
                }
            }
        }

        func cancel() {
            lock.withCriticalBlock {
                self.urlSessionTask?.cancel()
            }
        }

        func operation() -> MullvadRpc.Operation<Response> {
            return MullvadRpc.Operation(request: self)
        }

        private func makeURLRequest() -> Result<URLRequest, MullvadRpc.Error> {
            do {
                let data = try MullvadRpc.makeJSONEncoder().encode(request)

                return .success(Self.makeURLRequest(httpBody: data))
            } catch {
                return .failure(.encoding(error))
            }
        }

        private static func decodeResponse(_ responseData: Data) -> Result<Response, MullvadRpc.Error> {
            do {
                let serverResponse = try MullvadRpc.makeJSONDecoder()
                    .decode(JsonRpcResponse<Response, MullvadRpc.ResponseCode>.self, from: responseData)

                // unwrap JsonRpcResponse.result
                return serverResponse.result
                    .mapError { .server($0) }
            } catch {
                return .failure(.decoding(error))
            }
        }

        private static func makeURLRequest(httpBody: Data) -> URLRequest {
            var request = URLRequest(
                url: kMullvadAPIURL,
                cachePolicy: .useProtocolCachePolicy,
                timeoutInterval: kNetworkTimeout
            )
            request.addValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpMethod = "POST"
            request.httpBody = httpBody

            return request
        }
    }

    class Operation<Response>: AsyncOperation, InputOperation, OutputOperation where Response: Decodable {
        typealias Input = Request<Response>
        typealias Output = Result<Response, MullvadRpc.Error>

        init(request: Input? = nil) {
            super.init()
            self.input = request
        }

        override func main() {
            guard let request = self.input else {
                self.finish()
                return
            }

            request.start { [weak self] (result) in
                self?.finish(with: result)
            }
        }

        override func operationDidCancel() {
            input?.cancel()
        }
    }
}
