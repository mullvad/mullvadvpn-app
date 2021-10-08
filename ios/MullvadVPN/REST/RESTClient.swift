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
        static let shared = Client()

        private let session: URLSession
        private let sessionDelegate: SSLPinningURLSessionDelegate

        /// REST API v1 base URL
        private let baseURL = URL(string: "https://api.mullvad.net/app/v1")!

        /// Network request timeout in seconds
        private let networkTimeout: TimeInterval = 10

        /// Returns array of trusted root certificates
        private static var trustedRootCertificates: [SecCertificate] {
            let rootCertificate = Bundle.main.path(forResource: "le_root_cert", ofType: "cer")!

            return [rootCertificate].map { (path) -> SecCertificate in
                let data = FileManager.default.contents(atPath: path)!
                return SecCertificateCreateWithData(nil, data as CFData)!
            }
        }

        private init() {
            sessionDelegate = SSLPinningURLSessionDelegate(trustedRootCertificates: Self.trustedRootCertificates)
            session = URLSession(configuration: .ephemeral, delegate: sessionDelegate, delegateQueue: nil)
        }

        // MARK: - Public

        func createAccount() -> Result<AccountResponse, REST.Error>.Promise {
            let request = createURLRequest(method: .post, path: "accounts")

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    if httpResponse.statusCode == HTTPStatus.created {
                        return Self.decodeSuccessResponse(AccountResponse.self, from: data)
                    } else {
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
                }
        }

        func getRelays(etag: String?) -> Result<ServerRelaysCacheResponse, REST.Error>.Promise {
            var request = createURLRequest(method: .get, path: "relays")
            if let etag = etag {
                setETagHeader(etag: etag, request: &request)
            }

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
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
        }

        func getAccountExpiry(token: String) -> Result<AccountResponse, REST.Error>.Promise {
            var request = createURLRequest(method: .get, path: "me")

            setAuthenticationToken(token: token, request: &request)

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    if httpResponse.statusCode == HTTPStatus.ok {
                        return Self.decodeSuccessResponse(AccountResponse.self, from: data)
                    } else {
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
                }
        }

        func getWireguardKey(token: String, publicKey: PublicKey) -> Result<WireguardAddressesResponse, REST.Error>.Promise {
            let urlEncodedPublicKey = publicKey.base64Key
                .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

            let path = "wireguard-keys/".appending(urlEncodedPublicKey)
            var request = createURLRequest(method: .get, path: path)

            setAuthenticationToken(token: token, request: &request)

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    if httpResponse.statusCode == HTTPStatus.ok {
                        return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                    } else {
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
                }
        }

        func pushWireguardKey(token: String, publicKey: PublicKey) -> Result<WireguardAddressesResponse, REST.Error>.Promise {
            var request = createURLRequest(method: .post, path: "wireguard-keys")
            let body = PushWireguardKeyRequest(pubkey: publicKey.rawValue)

            setAuthenticationToken(token: token, request: &request)

            do {
                try setHTTPBody(value: body, request: &request)
            } catch {
                return .failure(.encodePayload(error))
            }

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    switch httpResponse.statusCode {
                    case .created, .ok:
                        return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                    default:
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
                }
        }

        func replaceWireguardKey(token: String, oldPublicKey: PublicKey, newPublicKey: PublicKey) -> Result<WireguardAddressesResponse, REST.Error>.Promise {
            var request = createURLRequest(method: .post, path: "replace-wireguard-key")
            let body = ReplaceWireguardKeyRequest(old: oldPublicKey.rawValue, new: newPublicKey.rawValue)

            setAuthenticationToken(token: token, request: &request)

            do {
                try setHTTPBody(value: body, request: &request)
            } catch {
                return .failure(.encodePayload(error))
            }

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    if httpResponse.statusCode == HTTPStatus.created {
                        return Self.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)
                    } else {
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
                }
        }

        func deleteWireguardKey(token: String, publicKey: PublicKey) -> Result<(), REST.Error>.Promise {
            let urlEncodedPublicKey = publicKey.base64Key
                .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

            let path = "wireguard-keys/".appending(urlEncodedPublicKey)
            var request = createURLRequest(method: .delete, path: path)

            setAuthenticationToken(token: token, request: &request)

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    if httpResponse.statusCode == HTTPStatus.noContent {
                        return .success(())
                    } else {
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
                }
        }

        func createApplePayment(token: String, receiptString: Data) -> Result<CreateApplePaymentResponse, REST.Error>.Promise {
            var request = createURLRequest(method: .post, path: "create-apple-payment")
            let body = CreateApplePaymentRequest(receiptString: receiptString)

            setAuthenticationToken(token: token, request: &request)

            do {
                try setHTTPBody(value: body, request: &request)
            } catch {
                return .failure(.encodePayload(error))
            }

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
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
        }

        func sendProblemReport(_ body: ProblemReportRequest) -> Result<(), REST.Error>.Promise {
            var request = createURLRequest(method: .post, path: "problem-report")

            do {
                try setHTTPBody(value: body, request: &request)
            } catch {
                return .failure(.encodePayload(error))
            }

            return dataTaskPromise(request: request)
                .mapError(self.mapNetworkError)
                .flatMap { httpResponse, data in
                    if httpResponse.statusCode == HTTPStatus.noContent {
                        return .success(())
                    } else {
                        return Self.decodeErrorResponseAndMapToServerError(from: data)
                    }
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

        private func dataTaskPromise(request: URLRequest) -> Result<(HTTPURLResponse, Data), URLError>.Promise {
            return Result<(HTTPURLResponse, Data), URLError>.Promise { resolver in
                let task = self.dataTask(request: request) { result in
                    resolver.resolve(value: result)
                }

                resolver.setCancelHandler {
                    task.cancel()
                }

                task.resume()
            }
        }

        private func setHTTPBody<T: Encodable>(value: T, request: inout URLRequest) throws {
            request.httpBody = try REST.Coding.makeJSONEncoder().encode(value)
        }

        private func setETagHeader(etag: String, request: inout URLRequest) {
            var etag = etag
            // Enforce weak validator to account for some backend caching quirks.
            if etag.starts(with: "\"") {
                etag.insert(contentsOf: "W/", at: etag.startIndex)
            }
            request.setValue(etag, forHTTPHeaderField: HTTPHeader.ifNoneMatch)
        }

        private func setAuthenticationToken(token: String, request: inout URLRequest) {
            request.addValue("Token \(token)", forHTTPHeaderField: HTTPHeader.authorization)
        }

        private func createURLRequest(method: HTTPMethod, path: String) -> URLRequest {
            var request = URLRequest(
                url: baseURL.appendingPathComponent(path),
                cachePolicy: .useProtocolCachePolicy,
                timeoutInterval: networkTimeout
            )
            request.httpShouldHandleCookies = false
            request.addValue("application/json", forHTTPHeaderField: HTTPHeader.contentType)
            request.httpMethod = method.rawValue
            return request
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
