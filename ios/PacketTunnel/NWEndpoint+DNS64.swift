//
//  DNSResolver.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import Network
import os

extension NWEndpoint {

    func withReresolvedIP() throws -> NWEndpoint {
        var resolvedAddress = self

        let hostname: String
        let port: NWEndpoint.Port

        if case .hostPort(let host, let hostPort) = self {
            hostname = "\(host)"
            port = hostPort
        } else {
            return resolvedAddress
        }

        var resultPointer = UnsafeMutablePointer<addrinfo>(OpaquePointer(bitPattern: 0))
        var hints = addrinfo(
            ai_flags: 0, // We set this to zero so that we actually resolve this using DNS64
            ai_family: AF_UNSPEC,
            ai_socktype: SOCK_DGRAM,
            ai_protocol: IPPROTO_UDP,
            ai_addrlen: 0,
            ai_canonname: nil,
            ai_addr: nil,
            ai_next: nil)

        let err = getaddrinfo(hostname, "\(port)", &hints, &resultPointer)
        if err != 0 || resultPointer == nil {
            throw NSError(
                domain: NSPOSIXErrorDomain,
                code: Int(err),
                userInfo: [
                    NSLocalizedDescriptionKey: String(cString: gai_strerror(err))
                ])
        }

        let result = resultPointer!.pointee
        if result.ai_family == AF_INET && result.ai_addrlen == MemoryLayout<sockaddr_in>.size {
            var sa4 = UnsafeRawPointer(result.ai_addr)!.assumingMemoryBound(to: sockaddr_in.self).pointee
            let addr = IPv4Address(Data(bytes: &sa4.sin_addr, count: MemoryLayout<in_addr>.size))

            resolvedAddress = NWEndpoint.hostPort(host: .ipv4(addr!), port: port)
        } else if result.ai_family == AF_INET6 && result.ai_addrlen == MemoryLayout<sockaddr_in6>.size {
            var sa6 = UnsafeRawPointer(result.ai_addr)!.assumingMemoryBound(to: sockaddr_in6.self).pointee
            let addr = IPv6Address(Data(bytes: &sa6.sin6_addr, count: MemoryLayout<in6_addr>.size))

            resolvedAddress = NWEndpoint.hostPort(host: .ipv6(addr!), port: port)
        }

        freeaddrinfo(resultPointer)

        if case .hostPort(let resolvedHost, _) = resolvedAddress {
            if "\(resolvedHost)" == hostname {
                os_log(.debug, "DNS64: mapped %{public}s to itself", "\(resolvedHost)")
            } else {
                os_log(.debug, "DNS64: mapped %{public}s to %{public}s", "\(resolvedHost)", "\(resolvedAddress)")
            }
        }

        return resolvedAddress
    }
}
