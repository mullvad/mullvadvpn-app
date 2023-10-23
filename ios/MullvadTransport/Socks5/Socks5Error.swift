//
//  Socks5Error.swift
//  MullvadTransport
//
//  Created by pronebird on 21/10/2023.
//

import Foundation
import Network

/// The errors returned by objects implementing socks proxy.
public enum Socks5Error: Error {
    /// Unexpected end of stream.
    case unexpectedEndOfStream

    /// Failure to decode the domain name from byte stream into utf8 string.
    case decodeDomainName

    /// Failure to parse IPv4 address from raw data.
    case parseIPv4Address

    /// Failure to parse IPv6 address from raw data.
    case parseIPv6Address

    /// Server replied with invalid socks version.
    case invalidSocksVersion

    /// Server replied with unknown endpoint address type.
    case invalidAddressType

    /// Invalid (unassigned) status code is returned.
    case invalidStatusCode(UInt8)

    /// Server replied with unsupported authentication method.
    case unsupportedAuthMethod

    /// None of the auth methods listed by the client are acceptable.
    case unacceptableAuthMethods

    /// Connection request is rejected.
    case connectionRejected(Socks5StatusCode)

    /// Failure to instantiate a TCP listener.
    case createTcpListener(Error)

    /// Socks forwarding proxy was cancelled during startup.
    case cancelledDuringStartup

    /// Local connection failure.
    case localConnectionFailure(NWError)

    /// Remote connection failure.
    case remoteConnectionFailure(NWError)
}
