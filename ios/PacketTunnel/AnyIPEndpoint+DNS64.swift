//
//  AnyIPEndpoint+DNS64.swift
//  PacketTunnel
//
//  Created by pronebird on 24/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//  Copyright © 2018-2019 WireGuard LLC. All Rights Reserved.
//

import Foundation
import Network

extension AnyIPEndpoint {

    /// Returns new `AnyIPEndpoint` resolved using DNS64
    func withReresolvedIP() -> Result<AnyIPEndpoint, Error> {
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

        let err = getaddrinfo("\(self.ip)", "\(self.port)", &hints, &resultPointer)
        if err != 0 || resultPointer == nil {
            return .failure(NSError(
                domain: NSPOSIXErrorDomain,
                code: Int(err),
                userInfo: [
                    NSLocalizedDescriptionKey: String(cString: gai_strerror(err))
            ]))
        }

        var resolvedAddress = self
        let result = resultPointer!.pointee
        if result.ai_family == AF_INET && result.ai_addrlen == MemoryLayout<sockaddr_in>.size {
            var sa4 = UnsafeRawPointer(result.ai_addr)!.assumingMemoryBound(to: sockaddr_in.self).pointee
            let addr = IPv4Address(Data(bytes: &sa4.sin_addr, count: MemoryLayout<in_addr>.size))

            resolvedAddress = .ipv4(IPv4Endpoint(ip: addr!, port: self.port))
        } else if result.ai_family == AF_INET6 && result.ai_addrlen == MemoryLayout<sockaddr_in6>.size {
            var sa6 = UnsafeRawPointer(result.ai_addr)!.assumingMemoryBound(to: sockaddr_in6.self).pointee
            let addr = IPv6Address(Data(bytes: &sa6.sin6_addr, count: MemoryLayout<in6_addr>.size))

            resolvedAddress = .ipv6(IPv6Endpoint(ip: addr!, port: self.port))
        }

        freeaddrinfo(resultPointer)

        return .success(resolvedAddress)
    }
}
