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
    /// Reference: https://developer.apple.com/support/ipv6/
    func withResolvedIP() -> Result<AnyIPEndpoint, Error> {
        var hints = addrinfo()
        hints.ai_family = PF_UNSPEC
        hints.ai_socktype = SOCK_DGRAM
        hints.ai_protocol = IPPROTO_UDP
        hints.ai_flags = AI_DEFAULT

        var result: UnsafeMutablePointer<addrinfo>?
        defer {
            result.flatMap { freeaddrinfo($0) }
        }

        let errorCode = getaddrinfo("\(self.ip)", "\(self.port)", &hints, &result)
        if errorCode != 0 {
            let userInfo = [
                NSLocalizedDescriptionKey: String(cString: gai_strerror(errorCode))
            ]
            let error = NSError(domain: NSPOSIXErrorDomain, code: Int(errorCode), userInfo: userInfo)

            return .failure(error)
        }

        let addrInfo = result!.pointee
        var endpoint: AnyIPEndpoint
        if let ipv4Address = IPv4Address(addrInfo: addrInfo) {
            endpoint = .ipv4(IPv4Endpoint(ip: ipv4Address, port: port))
        } else if let ipv6Address = IPv6Address(addrInfo: addrInfo) {
            endpoint = .ipv6(IPv6Endpoint(ip: ipv6Address, port: port))
        } else {
            fatalError()
        }

        return .success(endpoint)
    }
}

extension IPv4Address {
    init?(addrInfo: addrinfo) {
        guard addrInfo.ai_family == AF_INET else { return nil }

        let addressData = addrInfo.ai_addr.withMemoryRebound(to: sockaddr_in.self, capacity: MemoryLayout<sockaddr_in>.size) { (ptr) -> Data in
            return Data(bytes: &ptr.pointee.sin_addr, count: MemoryLayout<in_addr>.size)
        }

        if let ipAddress = IPv4Address(addressData) {
            self = ipAddress
        } else {
            return nil
        }
    }
}

extension IPv6Address {
    init?(addrInfo: addrinfo) {
        guard addrInfo.ai_family == AF_INET6 else { return nil }

        let addressData = addrInfo.ai_addr.withMemoryRebound(to: sockaddr_in6.self, capacity: MemoryLayout<sockaddr_in6>.size) { (ptr) -> Data in
            return Data(bytes: &ptr.pointee.sin6_addr, count: MemoryLayout<in6_addr>.size)
        }

        if let ipAddress = IPv6Address(addressData) {
            self = ipAddress
        } else {
            return nil
        }
    }
}
