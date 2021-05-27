//
//  MullvadRest.swift
//  MullvadVPN
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import Security
import WireGuardKit

/// REST API v1 base URL
private let kRestBaseURL = URL(string: "https://api.mullvad.net/app/v1")!

/// Network request timeout in seconds
private let kNetworkTimeout: TimeInterval = 10

/// HTTP method
enum HttpMethod: String {
    case get = "GET"
    case post = "POST"
    case delete = "DELETE"
}

// HTTP status codes
enum HttpStatus {
    static let ok = 200
    static let created = 201
    static let noContent = 204
    static let notModified = 304
}

/// HTTP headers
enum HttpHeader {
    static let authorization = "Authorization"
    static let contentType = "Content-Type"
    static let etag = "ETag"
    static let ifNoneMatch = "If-None-Match"
}

/// A struct that represents a server response in case of error (any HTTP status code except 2xx).
struct ServerErrorResponse: LocalizedError, Decodable, Equatable {
    /// A list of known server error codes
    enum Code: String, Equatable {
        case invalidAccount = "INVALID_ACCOUNT"
        case keyLimitReached = "KEY_LIMIT_REACHED"
        case pubKeyNotFound = "PUBKEY_NOT_FOUND"

        static func ~= (pattern: Self, value: ServerErrorResponse) -> Bool {
            return pattern.rawValue == value.code
        }
    }

    static var invalidAccount: Code {
        return .invalidAccount
    }
    static var keyLimitReached: Code {
        return .keyLimitReached
    }
    static var pubKeyNotFound: Code {
        return .pubKeyNotFound
    }

    let code: String
    let error: String?

    var errorDescription: String? {
        switch code {
        case Code.keyLimitReached.rawValue:
            return NSLocalizedString("Too many WireGuard keys in use.", comment: "")
        case Code.invalidAccount.rawValue:
            return NSLocalizedString("Invalid account.", comment: "")
        default:
            return nil
        }
    }

    var recoverySuggestion: String? {
        switch code {
        case Code.keyLimitReached.rawValue:
            return NSLocalizedString("Please visit the website to revoke a key before login is possible.", comment: "")
        default:
            return nil
        }
    }

    static func == (lhs: Self, rhs: Self) -> Bool {
        return lhs.code == rhs.code
    }
}

/// An error type returned by `MullvadRest`
enum RestError: ChainedError {
    /// A failure to encode the payload
    case encodePayload(Error)

    /// A failure during networking
    case network(URLError)

    /// A failure reported by server
    case server(ServerErrorResponse)

    /// A failure to decode the error response from server
    case decodeErrorResponse(Error)

    /// A failure to decode the success response from server
    case decodeSuccessResponse(Error)

    var errorDescription: String? {
        switch self {
        case .encodePayload:
            return "Failure to encode the payload"
        case .network:
            return "Network error"
        case .server:
            return "Server error"
        case .decodeErrorResponse:
            return "Failure to decode error response from server"
        case .decodeSuccessResponse:
            return "Failure to decode success response from server"
        }
    }
}

/// Types conforming to this protocol can participate in forming the `URLRequest` created by
/// `RestEndpoint`.
protocol RestPayload {
    func inject(into request: inout URLRequest) throws
}

/// Any `Encodable` type can be injected as JSON payload
extension RestPayload where Self: Encodable {
    func inject(into request: inout URLRequest) throws {
        request.httpBody = try MullvadRest.makeJSONEncoder().encode(self)
    }
}

// MARK: - Operations

final class RestOperation<Input, Response>: AsyncOperation, InputOperation, OutputOperation
    where Input: RestPayload
{
    typealias Output = Result<Response, RestError>

    private let endpoint: RestEndpoint<Input, Response>
    private let session: URLSession
    private var task: URLSessionTask?

    init(endpoint: RestEndpoint<Input, Response>, session: URLSession, input: Input? = nil) {
        self.endpoint = endpoint
        self.session = session

        super.init()
        self.input = input
    }

    override func main() {
        guard let payload = self.input else {
            finish()
            return
        }

        let result = endpoint.dataTask(session: session, payload: payload) { [weak self] (result) in
            self?.finish(with: result)
        }

        switch result {
        case .success(let task):
            self.task = task
            task.resume()
        case .failure(let error):
            finish(with: .failure(error))
        }
    }

    override func operationDidCancel() {
        task?.cancel()
        task = nil
    }
}

// MARK: - Response handlers

/// Types conforming to this protocol can be used as response handlers to decode the raw server response to the data
/// type expected by the caller.
protocol ResponseHandler {
    associatedtype Response

    /// Decode the response.
    /// The implementation is expected to throw `BadResponseError` in case of failure to handle the HTTP response,
    /// or any other `Error` in case of failure to decode the data.
    func decodeResponse(_ httpResponse: HTTPURLResponse, data: Data) -> Result<Response, ResponseHandlerError>
}

enum ResponseHandlerError: Error {
    /// A failure to handle the response due to unexpected status code
    case badResponse(_ statusCode: Int)

    /// A failure to decode data
    case decodeData(Error)
}

/// A placeholder error used to indicate that the server returned unexpected response.
fileprivate struct BadResponseError: Error {
    let statusCode: Int
}

/// Type-erasing response handler.
struct AnyResponseHandler<Response>: ResponseHandler {
    private let decodeResponseBlock: (HTTPURLResponse, Data) -> Result<Response, ResponseHandlerError>

    init<T: ResponseHandler>(_ wrappedHandler: T) where T.Response == Response {
        self.decodeResponseBlock = { (response, data) -> Result<Response, ResponseHandlerError> in
            return wrappedHandler.decodeResponse(response, data: data)
        }
    }

    init(block: @escaping (HTTPURLResponse, Data) -> Result<Response, ResponseHandlerError>) {
        self.decodeResponseBlock = block
    }

    func decodeResponse(_ httpResponse: HTTPURLResponse, data: Data) -> Result<Response, ResponseHandlerError> {
        return self.decodeResponseBlock(httpResponse, data)
    }
}

/// A REST response handler that decides when response contains the successful result based on the given status code and
/// decodes the value in response.
struct DecodingResponseHandler<Response: Decodable>: ResponseHandler {
    private let expectedStatus: Int

    init(expectedStatus: Int) {
        self.expectedStatus = expectedStatus
    }

    func decodeResponse(_ httpResponse: HTTPURLResponse, data: Data) -> Result<Response, ResponseHandlerError> {
        if httpResponse.statusCode == expectedStatus {
            return MullvadRest.decodeSuccessResponse(Response.self, from: data)
        } else {
            return .failure(.badResponse(httpResponse.statusCode))
        }
    }
}

/// A REST response handler that decides when response contains the successful result based on the given status code but
/// never decodes the value in response as it anticipates it to be empty.
struct EmptyResponseHandler: ResponseHandler {
    private let expectedStatus: Int

    init(expectedStatus: Int) {
        self.expectedStatus = expectedStatus
    }

    func decodeResponse(_ httpResponse: HTTPURLResponse, data: Data) -> Result<(), ResponseHandlerError> {
        if httpResponse.statusCode == expectedStatus {
            return .success(())
        } else {
            return .failure(.badResponse(httpResponse.statusCode))
        }
    }
}

/// A REST response handler that takes into account ETag and 200 and 304 response codes to produce the output result.
struct HttpCacheDecodingResponseHandler<WrappedType: Decodable>: ResponseHandler {
    typealias Response = HttpResourceCacheResponse<WrappedType>

    private let etag: String?

    init(etag: String?) {
        self.etag = etag
    }

    func decodeResponse(_ httpResponse: HTTPURLResponse, data: Data) -> Result<Response, ResponseHandlerError> {
        switch httpResponse.statusCode {
        case HttpStatus.ok:
            return MullvadRest.decodeSuccessResponse(WrappedType.self, from: data)
                .map { (relays) -> Response in
                    let etag = httpResponse.value(forCaseInsensitiveHTTPHeaderField: HttpHeader.etag)

                    return .newContent(etag, relays)
                }

        case HttpStatus.notModified where etag != nil:
            return .success(.notModified)

        case let statusCode:
            return .failure(.badResponse(statusCode))
        }
    }
}

// MARK: - Endpoints

/// A struct that describes the REST endpoint, including the expected input and output
struct RestEndpoint<Input, Response> where Input: RestPayload {
    let endpointURL: URL
    let httpMethod: HttpMethod
    let makeResponseHandler: (Input) -> AnyResponseHandler<Response>

    init<Handler: ResponseHandler>(endpointURL: URL, httpMethod: HttpMethod, responseHandlerFactory: @escaping (Input) -> Handler) where Handler.Response == Response {
        self.endpointURL = endpointURL
        self.httpMethod = httpMethod
        self.makeResponseHandler = { (input) -> AnyResponseHandler<Response> in
            return AnyResponseHandler(responseHandlerFactory(input))
        }
    }

    /// Create `URLSessionDataTask` that automatically parses the HTTP response and returns the
    /// expected response type or error upon completion.
    func dataTask(session: URLSession, payload: Input, completionHandler: @escaping (Result<Response, RestError>) -> Void) -> Result<URLSessionDataTask, RestError> {
        return makeURLRequest(payload: payload).map { (request) -> URLSessionDataTask in
            return session.dataTask(with: request) { (responseData, urlResponse, error) in
                let handler = self.makeResponseHandler(payload)
                let result = Self.handleURLResponse(urlResponse, data: responseData, error: error, responseHandler: handler)
                completionHandler(result)
            }
        }
    }

    /// Create `RestOperation` that automatically parses the response and sets the expected output
    /// type or error upon completion.
    func operation(session: URLSession, payload: Input?) -> RestOperation<Input, Response> {
        return RestOperation(endpoint: self, session: session, input: payload)
    }

    /// Create `URLRequest` that can be used to send an HTTP request
    private func makeURLRequest(payload: Input) -> Result<URLRequest, RestError> {
        var request = makeEndpointURLRequest()
        do {
            try payload.inject(into: &request)

            return .success(request)
        } catch {
            return .failure(.encodePayload(error))
        }
    }

    /// Create a boilerplate `URLRequest` before injecting the payload
    private func makeEndpointURLRequest() -> URLRequest {
        var request = URLRequest(
            url: endpointURL,
            cachePolicy: .useProtocolCachePolicy,
            timeoutInterval: kNetworkTimeout
        )
        request.httpShouldHandleCookies = false
        request.addValue("application/json", forHTTPHeaderField: HttpHeader.contentType)
        request.httpMethod = httpMethod.rawValue
        return request
    }

    /// A private HTTP response handler
    private static func handleURLResponse(_ urlResponse: URLResponse?, data: Data?, error: Error?, responseHandler: AnyResponseHandler<Response>) -> Result<Response, RestError> {
        if let error = error {
            let networkError = error as? URLError ?? URLError(.unknown)

            return .failure(.network(networkError))
        }

        guard let httpResponse = urlResponse as? HTTPURLResponse else {
            return .failure(.network(URLError(.unknown)))
        }

        let data = data ?? Data()

        return responseHandler.decodeResponse(httpResponse, data: data)
            .flatMapError { (error) -> Result<Response, RestError> in
                switch error {
                case .badResponse:
                    // Try decoding the server error response in case when unexpected response is returned
                    return MullvadRest.decodeErrorResponse(httpResponse: httpResponse, data: data)
                        .flatMap { (serverErrorResponse) -> Result<Response, RestError> in
                            return .failure(.server(serverErrorResponse))
                        }

                case .decodeData(let decodingError):
                    return .failure(.decodeSuccessResponse(decodingError))
                }
            }
    }
}

/// A convenience class for `RestEndpoint` that transparently provides it with the `URLSession`
struct RestSessionEndpoint<Input, Response> where Input: RestPayload {
    let session: URLSession
    let endpoint: RestEndpoint<Input, Response>

    init(session: URLSession, endpoint: RestEndpoint<Input, Response>) {
        self.session = session
        self.endpoint = endpoint
    }

    /// Create `URLSessionDataTask` that automatically parses the HTTP response and returns the
    /// expected response type or error upon completion.
    func dataTask(payload: Input, completionHandler: @escaping (Result<Response, RestError>) -> Void) -> Result<URLSessionDataTask, RestError> {
        return endpoint.dataTask(session: session, payload: payload, completionHandler: completionHandler)
    }

    /// Create `RestOperation` that automatically parses the response and sets the expected output
    /// type or error upon completion.
    func operation(payload: Input?) -> RestOperation<Input, Response> {
        return endpoint.operation(session: session, payload: payload)
    }
}

// MARK: - REST interface

class MullvadRest {
    let session: URLSession

    private let sessionDelegate: SSLPinningURLSessionDelegate

    /// Returns array of trusted root certificates
    private static var trustedRootCertificates: [SecCertificate] {
        let oldRootCertificate = Bundle.main.path(forResource: "old_le_root_cert", ofType: "cer")!
        let newRootCertificate = Bundle.main.path(forResource: "new_le_root_cert", ofType: "cer")!

        return [oldRootCertificate, newRootCertificate].map { (path) -> SecCertificate in
            let data = FileManager.default.contents(atPath: path)!
            return SecCertificateCreateWithData(nil, data as CFData)!
        }
    }

    init() {
        sessionDelegate = SSLPinningURLSessionDelegate(trustedRootCertificates: Self.trustedRootCertificates)
        session = URLSession(configuration: .ephemeral, delegate: sessionDelegate, delegateQueue: nil)
    }

    func createAccount() -> RestSessionEndpoint<EmptyPayload, AccountResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.createAccount())
    }

    func getRelays() -> RestSessionEndpoint<ETagPayload<EmptyPayload>, HttpResourceCacheResponse<ServerRelaysResponse>> {
        return RestSessionEndpoint(session: session, endpoint: Self.getRelays())
    }

    func getAccountExpiry() -> RestSessionEndpoint<TokenPayload<EmptyPayload>, AccountResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.getAccountExpiry())
    }

    func getWireguardKey() -> RestSessionEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, WireguardAddressesResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.getWireguardKey())
    }

    func pushWireguardKey() -> RestSessionEndpoint<TokenPayload<PushWireguardKeyRequest>, WireguardAddressesResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.pushWireguardKey())
    }

    func replaceWireguardKey() -> RestSessionEndpoint<TokenPayload<ReplaceWireguardKeyRequest>, WireguardAddressesResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.replaceWireguardKey())
    }

    func deleteWireguardKey() -> RestSessionEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, ()> {
        return RestSessionEndpoint(session: session, endpoint: Self.deleteWireguardKey())
    }

    func createApplePayment() -> RestSessionEndpoint<TokenPayload<CreateApplePaymentRequest>, CreateApplePaymentResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.createApplePayment())
    }

    func sendProblemReport() -> RestSessionEndpoint<ProblemReportRequest, ()> {
        return RestSessionEndpoint(session: session, endpoint: Self.sendProblemReport())
    }
}

extension MullvadRest {
    /// POST /v1/accounts
    static func createAccount() -> RestEndpoint<EmptyPayload, AccountResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("accounts"),
            httpMethod: .post,
            responseHandlerFactory: { (input) in
                return DecodingResponseHandler(expectedStatus: HttpStatus.created)
            }
        )
    }

    /// GET /v1/relays
    static func getRelays() -> RestEndpoint<ETagPayload<EmptyPayload>, HttpResourceCacheResponse<ServerRelaysResponse>> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("relays"),
            httpMethod: .get,
            responseHandlerFactory: { (input) in
                return HttpCacheDecodingResponseHandler(etag: input.etag)
            }
        )
    }

    /// GET /v1/me
    static func getAccountExpiry() -> RestEndpoint<TokenPayload<EmptyPayload>, AccountResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("me"),
            httpMethod: .get,
            responseHandlerFactory: { (input) in
                return DecodingResponseHandler(expectedStatus: HttpStatus.ok)
            }
        )
    }
    /// GET /v1/wireguard-keys/{pubkey}
    static func getWireguardKey() -> RestEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, WireguardAddressesResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("wireguard-keys"),
            httpMethod: .get,
            responseHandlerFactory: { (input) in
                return DecodingResponseHandler(expectedStatus: HttpStatus.ok)
            }
        )
    }

    /// POST /v1/wireguard-keys
    static func pushWireguardKey() -> RestEndpoint<TokenPayload<PushWireguardKeyRequest>, WireguardAddressesResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("wireguard-keys"),
            httpMethod: .post,
            responseHandlerFactory: { (input) in
                return AnyResponseHandler { (httpResponse, data) -> Result<WireguardAddressesResponse, ResponseHandlerError> in
                    switch httpResponse.statusCode {
                    case HttpStatus.ok, HttpStatus.created:
                        return MullvadRest.decodeSuccessResponse(WireguardAddressesResponse.self, from: data)

                    default:
                        return .failure(.badResponse(httpResponse.statusCode))
                    }
                }
            }
        )
    }

    /// POST /v1/replace-wireguard-key
    static func replaceWireguardKey() -> RestEndpoint<TokenPayload<ReplaceWireguardKeyRequest>, WireguardAddressesResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("replace-wireguard-key"),
            httpMethod: .post,
            responseHandlerFactory: { (input) in
                return DecodingResponseHandler(expectedStatus: HttpStatus.created)
            }
        )
    }

    /// DELETE /v1/wireguard-keys/{pubkey}
    static func deleteWireguardKey() -> RestEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, ()> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("wireguard-keys"),
            httpMethod: .delete,
            responseHandlerFactory: { (input) in
                return EmptyResponseHandler(expectedStatus: HttpStatus.noContent)
            }
        )
    }

    /// POST /v1/create-apple-payment
    static func createApplePayment() -> RestEndpoint<TokenPayload<CreateApplePaymentRequest>, CreateApplePaymentResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("create-apple-payment"),
            httpMethod: .post,
            responseHandlerFactory: { (input) in
                return AnyResponseHandler { (httpResponse, data) -> Result<CreateApplePaymentResponse, ResponseHandlerError> in
                    switch httpResponse.statusCode {
                    case HttpStatus.ok:
                        return MullvadRest.decodeSuccessResponse(CreateApplePaymentRawResponse.self, from: data)
                            .map { (response) in
                                return .noTimeAdded(response.newExpiry)
                            }

                    case HttpStatus.created:
                        return MullvadRest.decodeSuccessResponse(CreateApplePaymentRawResponse.self, from: data)
                            .map { (response) in
                                return .timeAdded(response.timeAdded, response.newExpiry)
                            }

                    default:
                        return .failure(.badResponse(httpResponse.statusCode))
                    }
                }
            }
        )
    }

    static func sendProblemReport() -> RestEndpoint<ProblemReportRequest, ()> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("problem-report"),
            httpMethod: .post,
            responseHandlerFactory: { (input) in
                return EmptyResponseHandler(expectedStatus: HttpStatus.noContent)
            }
        )
    }

    /// Returns a JSON encoder used by REST API
    static func makeJSONEncoder() -> JSONEncoder {
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        encoder.dateEncodingStrategy = .iso8601
        encoder.dataEncodingStrategy = .base64
        return encoder
    }

    /// Returns a JSON decoder used by REST API
    static func makeJSONDecoder() -> JSONDecoder {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        decoder.dateDecodingStrategy = .iso8601
        decoder.dataDecodingStrategy = .base64
        return decoder
    }

    /// A private helper that parses the JSON response into the given `Decodable` type.
    fileprivate static func decodeSuccessResponse<T: Decodable>(_ type: T.Type, from data: Data) -> Result<T, ResponseHandlerError> {
        return Result { try MullvadRest.makeJSONDecoder().decode(type, from: data) }
            .mapError { (error) -> ResponseHandlerError in
                return .decodeData(error)
            }
    }

    /// A private helper that parses the JSON response in case of error (Any HTTP code except 2xx)
    fileprivate static func decodeErrorResponse(httpResponse: HTTPURLResponse, data: Data) -> Result<ServerErrorResponse, RestError> {
        return Result { () -> ServerErrorResponse in
            return try MullvadRest.makeJSONDecoder().decode(ServerErrorResponse.self, from: data)
        }.mapError({ (error) -> RestError in
            return .decodeErrorResponse(error)
        })
    }
}


// MARK: - Payload types

/// A payload that adds the authentication token into HTTP Authorization header
struct TokenPayload<Payload: RestPayload>: RestPayload {
    let token: String
    let payload: Payload

    init(token: String, payload: Payload) {
        self.token = token
        self.payload = payload
    }

    func inject(into request: inout URLRequest) throws {
        request.addValue("Token \(token)", forHTTPHeaderField: HttpHeader.authorization)
        try payload.inject(into: &request)
    }
}

/// A payload that adds the public key into the URL path
struct PublicKeyPayload<Payload: RestPayload>: RestPayload {
    let pubKey: Data
    let payload: Payload

    init(pubKey: Data, payload: Payload) {
        self.pubKey = pubKey
        self.payload = payload
    }

    func inject(into request: inout URLRequest) throws {
        let pathComponent = pubKey.base64EncodedString()
            .addingPercentEncoding(withAllowedCharacters: .alphanumerics)!

        request.url = request.url?.appendingPathComponent(pathComponent)
        try payload.inject(into: &request)
    }
}

/// A payload that adds the ETag header to the request
struct ETagPayload<Payload: RestPayload>: RestPayload {
    let etag: String?
    let enforceWeakValidator: Bool
    let payload: Payload

    init(etag: String?, enforceWeakValidator: Bool, payload: Payload) {
        self.etag = etag
        self.enforceWeakValidator = enforceWeakValidator
        self.payload = payload
    }

    func inject(into request: inout URLRequest) throws {
        if var etag = etag {
            // Enforce weak validator to account for some backend caching quirks.
            if enforceWeakValidator && etag.starts(with: "\"") {
                etag.insert(contentsOf: "W/", at: etag.startIndex)
            }
            request.setValue(etag, forHTTPHeaderField: HttpHeader.ifNoneMatch)
        }
        try payload.inject(into: &request)
    }
}

/// An empty payload placeholder type.
/// Use it in places where the payload is not expected
struct EmptyPayload: RestPayload {
    init() {}
    func inject(into request: inout URLRequest) throws {}
}


// MARK: - Response types

struct AccountResponse: Decodable {
    let token: String
    let expires: Date
}

struct ServerLocation: Codable {
    let country: String
    let city: String
    let latitude: Double
    let longitude: Double
}

struct ServerRelay: Codable {
    let hostname: String
    let active: Bool
    let owned: Bool
    let location: String
    let provider: String
    let weight: Int32
    let ipv4AddrIn: IPv4Address
    let ipv6AddrIn: IPv6Address
    let publicKey: Data
    let includeInCountry: Bool
}

struct ServerWireguardTunnels: Codable {
    let ipv4Gateway: IPv4Address
    let ipv6Gateway: IPv6Address
    let portRanges: [ClosedRange<UInt16>]
    let relays: [ServerRelay]
}

private extension HTTPURLResponse {
    func value(forCaseInsensitiveHTTPHeaderField headerField: String) -> String? {
        if #available(iOS 13.0, *) {
            return self.value(forHTTPHeaderField: headerField)
        } else {
            for case let key as String in self.allHeaderFields.keys {
                if case .orderedSame = key.caseInsensitiveCompare(headerField) {
                    return self.allHeaderFields[key] as? String
                }
            }
            return nil
        }
    }
}

enum HttpResourceCacheResponse<T: Decodable> {
    case notModified
    case newContent(_ etag: String?, _ value: T)
}

struct ServerRelaysResponse: Codable {
    let locations: [String: ServerLocation]
    let wireguard: ServerWireguardTunnels
}

struct PushWireguardKeyRequest: Encodable, RestPayload {
    let pubkey: Data
}

struct WireguardAddressesResponse: Decodable {
    let id: String
    let pubkey: Data
    let ipv4Address: IPAddressRange
    let ipv6Address: IPAddressRange
}

struct ReplaceWireguardKeyRequest: Encodable, RestPayload {
    let old: Data
    let new: Data
}

struct CreateApplePaymentRequest: Encodable, RestPayload {
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

struct ProblemReportRequest: Encodable, RestPayload {
    let address: String
    let message: String
    let log: String
    let metadata: [String: String]
}
