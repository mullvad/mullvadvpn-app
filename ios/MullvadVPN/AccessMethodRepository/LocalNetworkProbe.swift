//
//  LocalNetworkProbe.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-01-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

struct LocalNetworkProbe {
    /// Does a best effort attempt to trigger the local network privacy alert.
    ///
    /// It works by sending a UDP datagram to the discard service (port 9) of every
    /// IP address associated with a broadcast-capable interface. This should
    /// trigger the local network privacy alert, assuming the alert hasn’t already
    /// been displayed for this app.
    ///
    /// This code takes a ‘best effort’. It handles errors by ignoring them. As
    /// such, there’s guarantee that it’ll actually trigger the alert.
    ///
    /// - note: iOS devices don’t actually run the discard service. I’m using it
    /// here because I need a port to send the UDP datagram to and port 9 is
    /// always going to be safe (either the discard service is running, in which
    /// case it will discard the datagram, or it’s not, in which case the TCP/IP
    /// stack will discard it).
    ///
    /// There should be a proper API for this (r. 69157424).
    ///
    /// For more background on this, see [Triggering the Local Network Privacy Alert](https://developer.apple.com/forums/thread/663768).
    func triggerLocalNetworkPrivacyAlert() {
        let sock4 = socket(AF_INET, SOCK_DGRAM, 0)
        guard sock4 >= 0 else { return }
        defer { close(sock4) }
        let sock6 = socket(AF_INET6, SOCK_DGRAM, 0)
        guard sock6 >= 0 else { return }
        defer { close(sock6) }

        let addresses = addressesOfDiscardServiceOnBroadcastCapableInterfaces()
        var message = [UInt8]("!".utf8)
        for address in addresses {
            address.withUnsafeBytes { buffer in
                let socketAddress = buffer.baseAddress!.assumingMemoryBound(to: sockaddr.self)
                let addressLength = socklen_t(buffer.count)
                let socket = socketAddress.pointee.sa_family == AF_INET ? sock4 : sock6
                _ = sendto(socket, &message, message.count, MSG_DONTWAIT, socketAddress, addressLength)
            }
        }
    }

    /// Returns the addresses of the discard service (port 9) on every
    /// broadcast-capable interface.
    ///
    /// Each array entry is contains either a `sockaddr_in` or `sockaddr_in6`.
    private func addressesOfDiscardServiceOnBroadcastCapableInterfaces() -> [Data] {
        var addrList: UnsafeMutablePointer<ifaddrs>?

        let err = getifaddrs(&addrList)
        guard err == 0, let start = addrList else { return [] }
        defer { freeifaddrs(start) }

        return sequence(first: start, next: { $0.pointee.ifa_next })
            .compactMap { interfaceAddress -> Data? in
                guard
                    (interfaceAddress.pointee.ifa_flags & UInt32(bitPattern: IFF_BROADCAST)) != 0,
                    let socketAddress = interfaceAddress.pointee.ifa_addr
                else { return nil }
                var result = Data(UnsafeRawBufferPointer(
                    start: socketAddress,
                    count: Int(socketAddress.pointee.sa_len)
                ))
                switch CInt(socketAddress.pointee.sa_family) {
                case AF_INET:
                    result.withUnsafeMutableBytes { buf in
                        let sin = buf.baseAddress!.assumingMemoryBound(to: sockaddr_in.self)
                        sin.pointee.sin_port = UInt16(9).bigEndian
                    }
                case AF_INET6:
                    result.withUnsafeMutableBytes { buf in
                        let sin6 = buf.baseAddress!.assumingMemoryBound(to: sockaddr_in6.self)
                        sin6.pointee.sin6_port = UInt16(9).bigEndian
                    }
                default:
                    return nil
                }
                return result
            }
    }
}
