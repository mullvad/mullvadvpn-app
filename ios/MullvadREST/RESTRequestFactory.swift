//
//  RESTRequestFactory.swift
//  MullvadREST
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension REST {
    final class RequestFactory {
        let hostname: String
        let pathPrefix: String
        let networkTimeout: TimeInterval
        let bodyEncoder: JSONEncoder

        class func withDefaultAPICredentials(
            pathPrefix: String,
            bodyEncoder: JSONEncoder
        ) -> RequestFactory {
            return RequestFactory(
                hostname: defaultAPIHostname,
                pathPrefix: pathPrefix,
                networkTimeout: defaultAPINetworkTimeout,
                bodyEncoder: bodyEncoder
            )
        }

        init(
            hostname: String,
            pathPrefix: String,
            networkTimeout: TimeInterval,
            bodyEncoder: JSONEncoder
        ) {
            self.hostname = hostname
            self.pathPrefix = pathPrefix
            self.networkTimeout = networkTimeout
            self.bodyEncoder = bodyEncoder
        }

        func createRequest(
            endpoint: AnyIPEndpoint,
            method: HTTPMethod,
            pathTemplate: URLPathTemplate
        ) throws -> REST.Request {
            var urlComponents = URLComponents()
            urlComponents.scheme = "https"
            urlComponents.path = pathPrefix
            urlComponents.host = "\(endpoint.ip)"
            urlComponents.port = Int(endpoint.port)

            let pathString = try pathTemplate.pathString()
            let requestURL = urlComponents.url!.appendingPathComponent(pathString)

            var request = URLRequest(
                url: requestURL,
                cachePolicy: .useProtocolCachePolicy,
                timeoutInterval: networkTimeout
            )
            request.httpShouldHandleCookies = false
            request.addValue(hostname, forHTTPHeaderField: HTTPHeader.host)
            request.addValue("application/json", forHTTPHeaderField: HTTPHeader.contentType)
            request.httpMethod = method.rawValue

            let prefixedPathTemplate = URLPathTemplate(stringLiteral: pathPrefix) + pathTemplate

            return REST.Request(
                urlRequest: request,
                pathTemplate: prefixedPathTemplate
            )
        }

        func createRequestBuilder(
            endpoint: AnyIPEndpoint,
            method: HTTPMethod,
            pathTemplate: URLPathTemplate
        ) throws -> RequestBuilder {
            let request = try createRequest(
                endpoint: endpoint,
                method: method,
                pathTemplate: pathTemplate
            )

            return RequestBuilder(
                restRequest: request,
                bodyEncoder: bodyEncoder
            )
        }
    }

    struct RequestBuilder {
        private var restRequest: REST.Request
        private let bodyEncoder: JSONEncoder

        init(restRequest: REST.Request, bodyEncoder: JSONEncoder) {
            self.restRequest = restRequest
            self.bodyEncoder = bodyEncoder
        }

        mutating func setHTTPBody<T: Encodable>(value: T) throws {
            restRequest.urlRequest.httpBody = try bodyEncoder.encode(value)
        }

        mutating func setETagHeader(etag: String) {
            var etag = etag
            // Enforce weak validator to account for some backend caching quirks.
            if etag.starts(with: "\"") {
                etag.insert(contentsOf: "W/", at: etag.startIndex)
            }
            restRequest.urlRequest.setValue(etag, forHTTPHeaderField: HTTPHeader.ifNoneMatch)
        }

        mutating func setAuthorization(_ authorization: REST.Authorization) {
            restRequest.urlRequest.addValue(
                "Bearer \(authorization)",
                forHTTPHeaderField: HTTPHeader.authorization
            )
        }

        func getRequest() -> REST.Request {
            return restRequest
        }
    }

    struct URLPathTemplate: ExpressibleByStringLiteral {
        enum Component {
            case literal(String)
            case placeholder(String)
        }

        enum Error: LocalizedError {
            /// Replacement value is not provided for placeholder.
            case noReplacement(_ name: String)

            /// Failure to perecent encode replacement value.
            case percentEncoding

            var errorDescription: String? {
                switch self {
                case let .noReplacement(placeholder):
                    return "Replacement is not provided for \(placeholder)."

                case .percentEncoding:
                    return "Failed to percent encode replacement value."
                }
            }
        }

        private var components: [Component]
        private var replacements = [String: String]()

        init(stringLiteral value: StringLiteralType) {
            let slashCharset = CharacterSet(charactersIn: "/")

            components = value.split(separator: "/").map { subpath -> Component in
                if subpath.hasPrefix("{"), subpath.hasSuffix("}") {
                    let name = String(subpath.dropFirst().dropLast())

                    return .placeholder(name)
                } else {
                    return .literal(
                        subpath.trimmingCharacters(in: slashCharset)
                    )
                }
            }
        }

        private init(components: [Component]) {
            self.components = components
        }

        mutating func addPercentEncodedReplacement(
            name: String,
            value: String,
            allowedCharacters: CharacterSet
        ) throws {
            let encoded = value.addingPercentEncoding(
                withAllowedCharacters: allowedCharacters
            )

            if let encoded = encoded {
                replacements[name] = encoded
            } else {
                throw Error.percentEncoding
            }
        }

        var templateString: String {
            var combinedString = ""

            for component in components {
                combinedString += "/"

                switch component {
                case let .literal(string):
                    combinedString += string
                case let .placeholder(name):
                    combinedString += "{\(name)}"
                }
            }

            return combinedString
        }

        func pathString() throws -> String {
            var combinedPath = ""

            for component in components {
                combinedPath += "/"

                switch component {
                case let .literal(string):
                    combinedPath += string

                case let .placeholder(name):
                    if let string = replacements[name] {
                        combinedPath += string
                    } else {
                        throw Error.noReplacement(name)
                    }
                }
            }

            return combinedPath
        }

        static func + (lhs: URLPathTemplate, rhs: URLPathTemplate) -> URLPathTemplate {
            return URLPathTemplate(components: lhs.components + rhs.components)
        }
    }
}
