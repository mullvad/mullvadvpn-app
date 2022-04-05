//
//  RESTClient.swift
//  MullvadVPN
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import class WireGuardKitTypes.PublicKey
import struct WireGuardKitTypes.IPAddressRange

extension REST {

    class Client {
        static let shared: Client = {
            return Client(addressCacheStore: AddressCache.Store.shared)
        }()

        /// URL session.
        private let session: URLSession

        /// URL session delegate.
        private let sessionDelegate: SSLPinningURLSessionDelegate

        /// REST API hostname.
        private let apiHostname = "api.mullvad.net"

        /// REST API base path.
        private let apiBasePath = "/app/v1"

        /// Network request timeout in seconds.
        private let networkTimeout: TimeInterval = 10

        /// Address cache store.
        private let addressCacheStore: AddressCache.Store

        /// Operation queue used for running network requests.
        private let operationQueue = OperationQueue()

        /// Network task counter.
        private var networkTaskCounter: UInt32 = 0

        /// Lock used for internal synchronization.
        private var nslock = NSLock()

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

        func createAccount(retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<AccountResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "create-account", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                let request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "accounts")

                let dataTask = self.dataTask(request: request) { responseResult in
                    let restResult = responseResult
                        .mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<AccountResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse(AccountResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func getAddressList(retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<[AnyIPEndpoint], REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "get-api-addrs", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                let request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: "api-addrs")

                let dataTask = self.dataTask(request: request) { responseResult in
                    let restResult = responseResult.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<[AnyIPEndpoint], REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse([AnyIPEndpoint].self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func getRelays(etag: String?, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<ServerRelaysCacheResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "get-relays", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: "relays")
                if let etag = etag {
                    Self.setETagHeader(etag: etag, request: &request)
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<ServerRelaysCacheResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse(ServerRelaysResponse.self, from: data)
                                    .map { serverRelays in
                                        let newEtag = httpResponse.value(forCaseInsensitiveHTTPHeaderField: HTTPHeader.etag)
                                        return .newContent(newEtag, serverRelays)
                                    }
                            } else if httpResponse.statusCode == HTTPStatus.notModified && etag != nil {
                                return .success(.notModified)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func getAccountExpiry(token: String, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<AccountResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "get-account-expiry", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: "me")

                Self.setAuthenticationToken(token: token, request: &request)

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<AccountResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse(AccountResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func getWireguardKey(token: String, publicKey: PublicKey, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<WireguardAddressesResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "get-wireguard-key", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                let urlEncodedPublicKey = publicKey.base64Key
                    .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

                let path = "wireguard-keys/".appending(urlEncodedPublicKey)
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .get, path: path)

                Self.setAuthenticationToken(token: token, request: &request)

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<WireguardAddressesResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }
                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func pushWireguardKey(token: String, publicKey: PublicKey, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<WireguardAddressesResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "push-wireguard-key", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "wireguard-keys")
                let body = PushWireguardKeyRequest(pubkey: publicKey.rawValue)

                Self.setAuthenticationToken(token: token, request: &request)

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<WireguardAddressesResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func replaceWireguardKey(token: String, oldPublicKey: PublicKey, newPublicKey: PublicKey, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<WireguardAddressesResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "replace-wireguard-key", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "replace-wireguard-key")
                let body = ReplaceWireguardKeyRequest(old: oldPublicKey.rawValue, new: newPublicKey.rawValue)

                Self.setAuthenticationToken(token: token, request: &request)

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<WireguardAddressesResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func deleteWireguardKey(token: String, publicKey: PublicKey, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<(), REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "delete-wireguard-key", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                let urlEncodedPublicKey = publicKey.base64Key
                    .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

                let path = "wireguard-keys/".appending(urlEncodedPublicKey)
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .delete, path: path)

                Self.setAuthenticationToken(token: token, request: &request)

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<(), REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return .success(())
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }

                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func createApplePayment(token: String, receiptString: Data, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<CreateApplePaymentResponse, REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "create-apple-payment", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "create-apple-payment")
                let body = CreateApplePaymentRequest(receiptString: receiptString)

                Self.setAuthenticationToken(token: token, request: &request)

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<CreateApplePaymentResponse, REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return REST.Client.decodeSuccessResponse(CreateApplePaymentRawResponse.self, from: data)
                                    .map { (response) in
                                        if response.timeAdded > 0 {
                                            return .timeAdded(response.timeAdded, response.newExpiry)
                                        } else {
                                            return .noTimeAdded(response.newExpiry)
                                        }
                                    }
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }
                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        func sendProblemReport(_ body: ProblemReportRequest, retryStrategy: REST.RetryStrategy, completionHandler: @escaping (OperationCompletion<(), REST.Error>) -> Void) -> Cancellable {
            return scheduleOperation(name: "send-problem-report", retryStrategy: retryStrategy, completionHandler: completionHandler) { endpoint, finishOperation in
                var request = self.createURLRequestWithEndpoint(endpoint: endpoint, method: .post, path: "problem-report")

                do {
                    try Self.setHTTPBody(value: body, request: &request)
                } catch {
                    return .failure(.encodePayload(error))
                }

                let dataTask = self.dataTask(request: request) { restResponse in
                    let restResult = restResponse.mapError(Self.mapNetworkError)
                        .flatMap { httpResponse, data -> Result<(), REST.Error> in
                            if HTTPStatus.isSuccess(httpResponse.statusCode) {
                                return .success(())
                            } else {
                                return Self.decodeErrorResponseAndMapToServerError(from: data)
                            }
                        }
                    finishOperation(restResult)
                }

                return .success(dataTask)
            }
        }

        // MARK: - Private

        private func nextTaskIdentifier() -> UInt32 {
            nslock.lock()
            let (partialValue, isOverflow) = networkTaskCounter.addingReportingOverflow(1)
            let nextValue = isOverflow ? 1 : partialValue
            networkTaskCounter = nextValue
            nslock.unlock()

            return nextValue
        }

        private func scheduleOperation<Response>(name: String, retryStrategy: REST.RetryStrategy, completionHandler: @escaping NetworkOperation<Response>.CompletionHandler, taskGenerator: @escaping NetworkOperation<Response>.Generator) -> Cancellable {
            let operation = NetworkOperation(
                taskIdentifier: nextTaskIdentifier(),
                name: name,
                networkTaskGenerator: taskGenerator,
                addressCacheStore: addressCacheStore,
                retryStrategy: retryStrategy,
                completionHandler: completionHandler
            )

            operationQueue.addOperation(operation)

            return operation
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

        /// Parse JSON response into the given `Decodable` type.
        private static func decodeSuccessResponse<T: Decodable>(_ type: T.Type, from data: Data) -> Result<T, REST.Error> {
            return Result { try REST.Coding.makeJSONDecoder().decode(type, from: data) }
            .mapError { error in
                return .decodeSuccessResponse(error)
            }
        }

        /// Parse JSON response in case of error (Any HTTP code except 2xx).
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

        private static func mapNetworkError(_ error: URLError) -> REST.Error {
            return .network(error)
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
