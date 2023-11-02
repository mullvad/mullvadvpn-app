//
//  Socks5EndpointReader.swift
//  MullvadTransport
//
//  Created by pronebird on 21/10/2023.
//

import Foundation
import MullvadTypes
import Network

/// The object reading the endpoint data from connection.
struct Socks5EndpointReader {
    /// Connection to the socks proxy.
    let connection: NWConnection

    /// The expected address type.
    let addressType: Socks5AddressType

    /// Completion handler called upon success.
    let onComplete: (Socks5Endpoint) -> Void

    /// Failure handler.
    let onFailure: (Error) -> Void

    /// Start reading endpoint from connection.
    func perform() {
        // The length of IPv4 address in bytes.
        let ipv4AddressLength = 4

        // The length of IPv6 address in bytes.
        let ipv6AddressLength = 16

        switch addressType {
        case .ipv4:
            readBoundAddressAndPortInner(addressLength: ipv4AddressLength)

        case .ipv6:
            readBoundAddressAndPortInner(addressLength: ipv6AddressLength)

        case .domainName:
            readBoundDomainNameLength { [self] domainLength in
                readBoundAddressAndPortInner(addressLength: domainLength)
            }
        }
    }

    private func readBoundAddressAndPortInner(addressLength: Int) {
        // The length of port in bytes.
        let portLength = MemoryLayout<UInt16>.size

        // The entire length of address + port
        let byteSize = addressLength + portLength

        connection.receive(exactLength: byteSize) { [self] addressData, _, _, error in
            if let error {
                onFailure(Socks5Error.remoteConnectionFailure(error))
            } else if let addressData {
                do {
                    let endpoint = try parseEndpoint(addressData: addressData, addressLength: addressLength)

                    onComplete(endpoint)
                } catch {
                    onFailure(error)
                }
            } else {
                onFailure(Socks5Error.unexpectedEndOfStream)
            }
        }
    }

    private func readBoundDomainNameLength(completion: @escaping (Int) -> Void) {
        // The length of domain length parameter in bytes.
        let domainLengthLength = MemoryLayout<UInt8>.size

        connection.receive(exactLength: domainLengthLength) { [self] data, _, _, error in
            if let error {
                onFailure(Socks5Error.remoteConnectionFailure(error))
            } else if let domainNameLength = data?.first {
                completion(Int(domainNameLength))
            } else {
                onFailure(Socks5Error.unexpectedEndOfStream)
            }
        }
    }

    private func parseEndpoint(addressData: Data, addressLength: Int) throws -> Socks5Endpoint {
        // The length of port in bytes.
        let portLength = MemoryLayout<UInt16>.size

        guard addressData.count == addressLength + portLength else { throw Socks5Error.unexpectedEndOfStream }

        // Read address bytes.
        let addressBytes = addressData[0 ..< addressLength]

        // Read port bytes.
        let port = addressData[addressLength...].withUnsafeBytes { buffer in
            let value = buffer.load(as: UInt16.self)

            // Port is passed in network byte order. Convert it to host order.
            return UInt16(bigEndian: value)
        }

        // Parse address into endpoint.
        switch addressType {
        case .ipv4:
            guard let ipAddress = IPv4Address(addressBytes) else { throw Socks5Error.parseIPv4Address }

            return .ipv4(IPv4Endpoint(ip: ipAddress, port: port))

        case .ipv6:
            guard let ipAddress = IPv6Address(addressBytes) else { throw Socks5Error.parseIPv6Address }

            return .ipv6(IPv6Endpoint(ip: ipAddress, port: port))

        case .domainName:
            guard let hostname = String(bytes: addressBytes, encoding: .utf8),
                  let endpoint = Socks5HostEndpoint(hostname: hostname, port: port) else {
                throw Socks5Error.decodeDomainName
            }
            return .domain(endpoint)
        }
    }
}
