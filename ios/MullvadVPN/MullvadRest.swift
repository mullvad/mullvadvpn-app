//
//  MullvadRest.swift
//  MullvadVPN
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
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

/// A struct that represents a server response in case of error (any HTTP status code except 2xx).
struct ServerErrorResponse: LocalizedError, Decodable, RestResponse, Equatable {
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

/// Types conforming to this protocol can act as REST response types.
protocol RestResponse {
    associatedtype Output

    static func decodeResponse(_ data: Data) throws -> Output
}

/// Any `Decodable` can be REST response
extension Decodable where Self: RestResponse {
    static func decodeResponse(_ data: Data) throws -> Self {
        try MullvadRest.makeJSONDecoder().decode(Self.self, from: data)
    }
}

/// An empty REST response type that cannot be instantiated and is only used to produce an empty
/// output.
enum EmptyResponse {}
extension EmptyResponse: RestResponse {
    static func decodeResponse(_ data: Data) throws -> () {
        return ()
    }
}

/// Any `Encodable` type can be injected as JSON payload
extension RestPayload where Self: Encodable {
    func inject(into request: inout URLRequest) throws {
        request.httpBody = try MullvadRest.makeJSONEncoder().encode(self)
    }
}

// MARK: - Operations

final class RestOperation<Input, Response>: AsyncOperation, InputOperation, OutputOperation
    where Input: RestPayload, Response: RestResponse
{
    typealias Output = Result<Response.Output, RestError>

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

// MARK: - Endpoints

/// A struct that describes the REST endpoint, including the expected input and output
struct RestEndpoint<Input, Response> where Input: RestPayload, Response: RestResponse {
    let endpointURL: URL
    let httpMethod: HttpMethod

    init(endpointURL: URL, httpMethod: HttpMethod) {
        self.endpointURL = endpointURL
        self.httpMethod = httpMethod
    }

    /// Create `URLSessionDataTask` that automatically parses the HTTP response and returns the
    /// expected response type or error upon completion.
    func dataTask(session: URLSession, payload: Input, completionHandler: @escaping (Result<Response.Output, RestError>) -> Void) -> Result<URLSessionDataTask, RestError> {
        return makeURLRequest(payload: payload).map { (request) -> URLSessionDataTask in
            return session.dataTask(with: request) { (responseData, urlResponse, error) in
                let result = Self.handleURLResponse(urlResponse, data: responseData, error: error)
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
        request.addValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpMethod = httpMethod.rawValue
        return request
    }

    /// A private HTTP response handler
    private static func handleURLResponse(_ urlResponse: URLResponse?, data: Data?, error: Error?) -> Result<Response.Output, RestError> {
        if let error = error {
            let networkError = error as? URLError ?? URLError(.unknown)

            return .failure(.network(networkError))
        }

        guard let httpResponse = urlResponse as? HTTPURLResponse else {
            return .failure(.network(URLError(.unknown)))
        }

        let data = data ?? Data()

        // Treat all 2xx responses as success despite the subtle meaning they may convey
        if (200..<300).contains(httpResponse.statusCode) {
            return Self.decodeSuccessResponse(data)
        } else {
            return Self.decodeErrorResponse(data)
                .flatMap { (serverErrorResponse) -> Result<Response.Output, RestError> in
                    return .failure(.server(serverErrorResponse))
            }
        }
    }

    /// A private helper that parses the JSON response in case of success (HTTP 2xx)
    private static func decodeSuccessResponse(_ responseData: Data) -> Result<Response.Output, RestError> {
        return Result { () -> Response.Output in
            return try Response.decodeResponse(responseData)
        }.mapError({ (error) -> RestError in
            return .decodeSuccessResponse(error)
        })
    }

    /// A private helper that parses the JSON response in case of error (Any HTTP code except 2xx)
    private static func decodeErrorResponse(_ responseData: Data) -> Result<ServerErrorResponse, RestError> {
        return Result { () -> ServerErrorResponse in
            return try ServerErrorResponse.decodeResponse(responseData)
        }.mapError({ (error) -> RestError in
            return .decodeErrorResponse(error)
        })
    }
}

/// A convenience class for `RestEndpoint` that transparently provides it with the `URLSession`
struct RestSessionEndpoint<Input, Response> where Input: RestPayload, Response: RestResponse {
    let session: URLSession
    let endpoint: RestEndpoint<Input, Response>

    init(session: URLSession, endpoint: RestEndpoint<Input, Response>) {
        self.session = session
        self.endpoint = endpoint
    }

    /// Create `URLSessionDataTask` that automatically parses the HTTP response and returns the
    /// expected response type or error upon completion.
    func dataTask(payload: Input, completionHandler: @escaping (Result<Response.Output, RestError>) -> Void) -> Result<URLSessionDataTask, RestError> {
        return endpoint.dataTask(session: session, payload: payload, completionHandler: completionHandler)
    }

    /// Create `RestOperation` that automatically parses the response and sets the expected output
    /// type or error upon completion.
    func operation(payload: Input?) -> RestOperation<Input, Response> {
        return endpoint.operation(session: session, payload: payload)
    }
}

// MARK: - REST interface

struct MullvadRest {
    let session: URLSession

    init(session: URLSession = URLSession(configuration: .ephemeral)) {
        self.session = session
    }

    func createAccount() -> RestSessionEndpoint<EmptyPayload, AccountResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.createAccount())
    }

    func getRelays() -> RestSessionEndpoint<EmptyPayload, ServerRelaysResponse> {
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

    func deleteWireguardKey() -> RestSessionEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, EmptyResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.deleteWireguardKey())
    }

    func createApplePayment() -> RestSessionEndpoint<TokenPayload<CreateApplePaymentRequest>, CreateApplePaymentResponse> {
        return RestSessionEndpoint(session: session, endpoint: Self.createApplePayment())
    }
}

extension MullvadRest {
    /// POST /v1/accounts
    static func createAccount() -> RestEndpoint<EmptyPayload, AccountResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("accounts"),
            httpMethod: .post
        )
    }

    /// GET /v1/relays
    static func getRelays() -> RestEndpoint<EmptyPayload, ServerRelaysResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("relays"),
            httpMethod: .get
        )
    }

    /// GET /v1/me
    static func getAccountExpiry() -> RestEndpoint<TokenPayload<EmptyPayload>, AccountResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("me"),
            httpMethod: .get
        )
    }
    /// GET /v1/wireguard-keys/{pubkey}
    static func getWireguardKey() -> RestEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, WireguardAddressesResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("wireguard-keys"),
            httpMethod: .get
        )
    }

    /// POST /v1/wireguard-keys
    static func pushWireguardKey() -> RestEndpoint<TokenPayload<PushWireguardKeyRequest>, WireguardAddressesResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("wireguard-keys"),
            httpMethod: .post
        )
    }

    /// POST /v1/replace-wireguard-key
    static func replaceWireguardKey() -> RestEndpoint<TokenPayload<ReplaceWireguardKeyRequest>, WireguardAddressesResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("replace-wireguard-key"),
            httpMethod: .post
        )
    }

    /// DELETE /v1/wireguard-keys/{pubkey}
    static func deleteWireguardKey() -> RestEndpoint<PublicKeyPayload<TokenPayload<EmptyPayload>>, EmptyResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("wireguard-keys"),
            httpMethod: .delete
        )
    }

    /// POST /v1/create-apple-payment
    static func createApplePayment() -> RestEndpoint<TokenPayload<CreateApplePaymentRequest>, CreateApplePaymentResponse> {
        return RestEndpoint(
            endpointURL: kRestBaseURL.appendingPathComponent("create-apple-payment"),
            httpMethod: .post
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
        request.addValue("Token \(token)", forHTTPHeaderField: "Authorization")
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

/// An empty payload placeholder type.
/// Use it in places where the payload is not expected
struct EmptyPayload: RestPayload {
    init() {}
    func inject(into request: inout URLRequest) throws {}
}


// MARK: - Response types

struct AccountResponse: Decodable, RestResponse {
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

struct ServerRelaysResponse: Codable, RestResponse {
    let locations: [String: ServerLocation]
    let wireguard: ServerWireguardTunnels
}

struct PushWireguardKeyRequest: Encodable, RestPayload {
    let pubkey: Data
}

struct WireguardAddressesResponse: Decodable, RestResponse {
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

struct CreateApplePaymentResponse: Decodable, RestResponse {
    let timeAdded: Int
    let newExpiry: Date

    /// Returns a formatted string for the `timeAdded` interval, i.e "30 days"
    var formattedTimeAdded: String? {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour]
        formatter.unitsStyle = .full

        return formatter.string(from: TimeInterval(timeAdded))
    }
}
