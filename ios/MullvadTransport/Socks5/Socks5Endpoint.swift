//
//  Socks5Endpoint.swift
//  MullvadTransport
//
//  Created by pronebird on 20/10/2023.
//

import Foundation
import MullvadTypes
import Network

/// A network endpoint specified by DNS name and port.
public struct Socks5HostEndpoint {
    /// The endpoint's hostname.
    public let hostname: String

    /// The endpoint's port.
    public let port: UInt16

    /**
     Initializes a new host endpoint.

     Returns `nil` when the hostname is either empty or longer than 255 bytes, because it cannot be represented in socks protocol.

     - Parameters:
        - hostname: the endpoint's hostname
        - port: the endpoint's port
     */
    public init?(hostname: String, port: UInt16) {
        // The maximum length of domain name in bytes.
        let maxHostnameLength = UInt8.max
        let hostnameByteLength = Data(hostname.utf8).count

        // Empty hostname is meaningless.
        guard hostnameByteLength > 0 else { return nil }

        // The length larger than 255 bytes cannot be represented in socks protocol.
        guard hostnameByteLength <= maxHostnameLength else { return nil }

        self.hostname = hostname
        self.port = port
    }
}

/// The endpoint type used by objects implementing socks protocol.
public enum Socks5Endpoint {
    /// IPv4 endpoint.
    case ipv4(IPv4Endpoint)

    /// IPv6 endpoint.
    case ipv6(IPv6Endpoint)

    /// Domain name endpoint.
    case domain(Socks5HostEndpoint)

    /// The corresponding raw socks address type.
    var addressType: Socks5AddressType {
        switch self {
        case .ipv4:
            return .ipv4
        case .ipv6:
            return .ipv6
        case .domain:
            return .domainName
        }
    }

    /// The port associated with the underlying endpoint.
    var port: UInt16 {
        switch self {
        case let .ipv4(endpoint):
            endpoint.port
        case let .ipv6(endpoint):
            endpoint.port
        case let .domain(endpoint):
            endpoint.port
        }
    }

    /// The byte representation in socks protocol.
    var rawData: Data {
        var data = Data()

        switch self {
        case let .ipv4(endpoint):
            data.append(contentsOf: endpoint.ip.rawValue)

        case let .ipv6(endpoint):
            data.append(contentsOf: endpoint.ip.rawValue)

        case let .domain(endpoint):
            // Convert hostname to byte data without nul terminator.
            let domainNameBytes = Data(endpoint.hostname.utf8)

            // Append the length of domain name.
            // Host endpoint already ensures that the length of domain name does not exceed the maximum value that
            // single byte can hold.
            data.append(UInt8(domainNameBytes.count))

            // Append the domain name.
            data.append(contentsOf: domainNameBytes)
        }

        // Append port in network byte order.
        withUnsafeBytes(of: port.bigEndian) { buffer in
            data.append(contentsOf: buffer)
        }

        return data
    }
}
