//
//  RESTClient.swift
//  MullvadVPN
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import class WireGuardKit.PublicKey
import struct WireGuardKit.IPAddressRange

extension REST {

    class Client {
        static let shared: Client = {
            return Client(addressCacheStore: AddressCache.Store.shared)
        }()

        /// URL session
        private let session: URLSession

        /// URL session delegate
        private let sessionDelegate: SSLPinningURLSessionDelegate

        /// REST API hostname
        private let apiHostname = "api.mullvad.net"

        /// REST API base path
        private let apiBasePath = "/app/v1"

        /// Network request timeout in seconds
        private let networkTimeout: TimeInterval = 10

        /// Address cache store
        private let addressCacheStore: AddressCache.Store

        /// Operation queue used for running network requests
        private let operationQueue = OperationQueue()

        /// Returns array of trusted root certificates
        private static var trustedRootCertificates: [SecCertificate] {
            let rootCertificate = Bundle.main.path(forResource: "le_root_cert", ofType: "cer")!

            return [rootCertificate].map { (path) -> SecCertificate in
                let data = FileManager.default.contents(atPath: path)!
                return SecCertificateCreateWithData(nil, data as CFData)!
            }
        }

        init(addressCacheStore: AddressCache.Store) {
            sessionDelegate = SSLPinningURLSessionDelegate(sslHostname: apiHostname, trustedRootCertificates: Self.trustedRootCertificates)
            session = URLSession(configuration: .ephemeral, delegate: sessionDelegate, delegateQueue: nil)
            self.addressCacheStore = addressCacheStore
        }

        // MARK: - Public

        func createAccount() -> REST.RequestAdapter<AccountResponse> {
            return makeAdapter { endpoint, completionHandler in
                let request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "accounts")

                let dataTask = self.dataTask(request: request) { responseResult in
                    let restResult = responseResult
                        .mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<AccountResponse, REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.created {
                                return Self.decodeSuccessResponse(AccountResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func getAddressList() -> REST.RequestAdapter<[AnyIPEndpoint]> {
            return makeAdapter { endpoint, completionHandler in
                let request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: "api-addrs")

                let dataTask = self.dataTask(request: request) { responseResult in
                    let restResult = responseResult.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<[AnyIPEndpoint], REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.ok {
                                return Self.decodeSuccessResponse([AnyIPEndpoint].self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func getRelays(etag: String?) -> REST.RequestAdapter<ServerRelaysCacheResponse> {
            return makeAdapter { endpoint, completionHandler in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: "relays")
                if let etag = etag {
                    Self.setETagHeader(etag: etag, request: &request)
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<ServerRelaysCacheResponse, REST.Error> in
                            switch httpResponse.statusCode {
                            case .ok:
                                return Self.decodeSuccessResponse(ServerRelaysResponse.self, from: data)
                                    .map { serverRelays in
                                        let newEtag = httpResponse.value(forCaseInsensitiveHTTPHeaderField: HTTPHeader.etag)
                                        return .newContent(newEtag, serverRelays)
                                    }

                            case .notModified where etag != nil:
                                return .success(.notModified)

                            default:
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func getAccountExpiry(token: String) -> REST.RequestAdapter<AccountResponse> {
            return makeAdapter { endpoint, completionHandler in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: "me")

                Self.setAuthenticationToken(token: token, request: &request)

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<AccountResponse, REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.ok {
                                return Self.decodeSuccessResponse(AccountResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func getWireguardKey(token: String, publicKey: PublicKey) -> REST.RequestAdapter<WireguardAddressesResponse> {
            return makeAdapter { endpoint, completionHandler in
                let urlEncodedPublicKey = publicKey.base64Key
                    .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

                let path = "wireguard-keys/".appending(urlEncodedPublicKey)
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: path)

                Self.setAuthenticationToken(token: token, request: &request)

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<WireguardAddressesResponse, REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.ok {
                                return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }
                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func pushWireguardKey(token: String, publicKey: PublicKey) -> REST.RequestAdapter<WireguardAddressesResponse> {
            return makeAdapter { endpoint, completionHandler in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "wireguard-keys")
                let body = PushWireguardKeyRequest(pubkey: publicKey.rawValue)

                Self.setAuthenticationToken(token: token, request: &request)

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<WireguardAddressesResponse, REST.Error> in
                            switch httpResponse.statusCode {
                            case .created, .ok:
                                return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                            default:
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }


        func replaceWireguardKey(token: String, oldPublicKey: PublicKey, newPublicKey: PublicKey) -> REST.RequestAdapter<WireguardAddressesResponse> {
            return makeAdapter { endpoint, completionHandler in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "replace-wireguard-key")
                let body = ReplaceWireguardKeyRequest(old: oldPublicKey.rawValue, new: newPublicKey.rawValue)

                Self.setAuthenticationToken(token: token, request: &request)

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<WireguardAddressesResponse, REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.created {
                                return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func deleteWireguardKey(token: String, publicKey: PublicKey) -> REST.RequestAdapter<()> {
            return makeAdapter { endpoint, completionHandler in
                let urlEncodedPublicKey = publicKey.base64Key
                    .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

                let path = "wireguard-keys/".appending(urlEncodedPublicKey)
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .delete, path: path)

                Self.setAuthenticationToken(token: token, request: &request)

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<(), REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.noContent {
                                return .success(())
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func createApplePayment(token: String, receiptString: Data) -> REST.RequestAdapter<CreateApplePaymentResponse> {
            return makeAdapter { endpoint, completionHandler in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "create-apple-payment")
                let body = CreateApplePaymentRequest(receiptString: receiptString)

                Self.setAuthenticationToken(token: token, request: &request)

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<CreateApplePaymentResponse, REST.Error> in
                            switch httpResponse.statusCode {
                            case HTTPStatus.ok:
                                return REST.Client.decodeSuccessResponse(CreateApplePaymentRawResponse.self, from: data)
                                    .map { (response) in
                                        return .noTimeAdded(response.newExpiry)
                                    }

                            case HTTPStatus.created:
                                return REST.Client.decodeSuccessResponse(CreateApplePaymentRawResponse.self, from: data)
                                    .map { (response) in
                                        return .timeAdded(response.timeAdded, response.newExpiry)
                                    }

                            default:
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }
                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        func sendProblemReport(_ body: ProblemReportRequest) -> REST.RequestAdapter<()> {
            return makeAdapter { endpoint, completionHandler in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "problem-report")

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<(), REST.Error> in
                            if httpResponse.statusCode == HTTPStatus.noContent {
                                return .success(())
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }
                    completionHandler(restResult)
                }

                return .success(dataTask)
            }
        }

        // MARK: - Private

        /// A private helper that parses the JSON response into the given `Decodable` type.
        private static func decodeSuccessResponse<T: Decodable>(_ type: T.Type, from data: Data) -> Result<T, REST.Error> {
            return Result { try REST.Coding.makeJSONDecoder().decode(type, from: data) }
            .mapError { error in
                return .decodeSuccessResponse(error)
            }
        }

        /// A private helper that parses the JSON response in case of error (Any HTTP code except 2xx)
        private static func decodeErrorResponse(from data: Data) -> Result<ServerErrorResponse, REST.Error> {
            return Result { () -> ServerErrorResponse in
                return try REST.Coding.makeJSONDecoder().decode(ServerErrorResponse.self, from: data)
            }.mapError { error in
                return .decodeErrorResponse(error)
            }
        }

        private static func decodeErrorResponseAndMapToServerError<T>(from data: Data) -> Result<T, REST.Error> {
            return Self.decodeErrorResponse(from: data)
                .flatMap { serverError in
                    return .failure(.server(serverError))
                }
        }

        private func mapNetworkError(_ error: URLError) -> REST.Error {
            return .network(error)
        }

        private func dataTask(request: URLRequest, completion: @escaping (Result<(HTTPURLResponse, Data), URLError>) -> Void) -> URLSessionDataTask {
            return self.session.dataTask(with: request) { data, response, error in
                if let error = error {
                    let urlError = error as? URLError ?? URLError(.unknown)

                    completion(.failure(urlError))
                } else {
                    if let httpResponse = response as? HTTPURLResponse {
                        let data = data ?? Data()
                        let value = (httpResponse, data)

                        completion(.success(value))
                    } else {
                        completion(.failure(URLError(.unknown)))
                    }
                }
            }
        }

        private static func setHTTPBody<T: Encodable>(value: T, request: inout URLRequest) throws {
            request.httpBody = try REST.Coding.makeJSONEncoder().encode(value)
        }

        private static func setETagHeader(etag: String, request: inout URLRequest) {
            var etag = etag
            // Enforce weak validator to account for some backend caching quirks.
            if etag.starts(with: "\"") {
                etag.insert(contentsOf: "W/", at: etag.startIndex)
            }
            request.setValue(etag, forHTTPHeaderField: HTTPHeader.ifNoneMatch)
        }

        private static func setAuthenticationToken(token: String, request: inout URLRequest) {
            request.addValue("Token \(token)", forHTTPHeaderField: HTTPHeader.authorization)
        }

        private func createURLRequestWithEndpoint(endpoint: AnyIPEndpoint, method: HTTPMethod, path: String) -> URLRequest {
            var urlComponents = URLComponents()
            urlComponents.scheme = "https"
            urlComponents.path = apiBasePath
            urlComponents.host = "\(endpoint.ip)"
            urlComponents.port = Int(endpoint.port)

            let requestURL = urlComponents.url!.appendingPathComponent(path)

            var request = URLRequest(
                url: requestURL,
                cachePolicy: .useProtocolCachePolicy,
                timeoutInterval: networkTimeout
            )
            request.httpShouldHandleCookies = false
            request.addValue(apiHostname, forHTTPHeaderField: HTTPHeader.host)
            request.addValue("application/json", forHTTPHeaderField: HTTPHeader.contentType)
            request.httpMethod = method.rawValue
            return request
        }

        private func makeAdapter<T>(_ networkTaskGenerator: @escaping NetworkOperation<T>.Generator) -> REST.RequestAdapter<T> {
            return REST.RequestAdapter { retryStrategy, completionHandler in
                let operation = NetworkOperation(
                    networkTaskGenerator: networkTaskGenerator,
                    addressCacheStore: self.addressCacheStore,
                    retryStrategy: retryStrategy,
                    completionHandler: completionHandler
                )

                self.operationQueue.addOperation(operation)

                return AnyCancellable {
                    operation.cancel()
                }
            }
        }

    }

    // MARK: - Response types

    struct AccountResponse: Decodable {
        let token: String
        let expires: Date
    }

    enum ServerRelaysCacheResponse {
        case notModified
        case newContent(_ etag: String?, _ value: ServerRelaysResponse)
    }

    struct WireguardAddressesResponse: Decodable {
        let id: String
        let pubkey: Data
        let ipv4Address: IPAddressRange
        let ipv6Address: IPAddressRange
    }

    fileprivate struct PushWireguardKeyRequest: Encodable {
        let pubkey: Data
    }

    fileprivate struct ReplaceWireguardKeyRequest: Encodable {
        let old: Data
        let new: Data
    }

    fileprivate struct CreateApplePaymentRequest: Encodable {
        let receiptString: Data
    }

    enum CreateApplePaymentResponse {
        case noTimeAdded(_ expiry: Date)
        case timeAdded(_ timeAdded: Int, _ newExpiry: Date)

        var newExpiry: Date {
            switch self {
            case .noTimeAdded(let expiry), .timeAdded(_, let expiry):
                return expiry
            }
        }

        var timeAdded: TimeInterval {
            switch self {
            case .noTimeAdded:
                return 0
            case .timeAdded(let timeAdded, _):
                return TimeInterval(timeAdded)
            }
        }

        /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
        var formattedTimeAdded: String? {
            let formatter = DateComponentsFormatter()
            formatter.allowedUnits = [.day, .hour]
            formatter.unitsStyle = .full

            return formatter.string(from: self.timeAdded)
        }
    }

    fileprivate struct CreateApplePaymentRawResponse: Decodable {
        let timeAdded: Int
        let newExpiry: Date
    }

    struct ProblemReportRequest: Encodable {
        let address: String
        let message: String
        let log: String
        let metadata: [String: String]
    }

}
