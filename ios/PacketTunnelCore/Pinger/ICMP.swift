//
//  ICMP.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-07-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct ICMP {
    public enum Error: LocalizedError {
        case malformedResponse(MalformedResponseReason)

        public var errorDescription: String? {
            switch self {
            case let .malformedResponse(reason):
                return "Malformed response: \(reason)."
            }
        }
    }

    public enum MalformedResponseReason {
        case ipv4PacketTooSmall
        case icmpHeaderTooSmall
        case invalidIPVersion
        case checksumMismatch(UInt16, UInt16)
    }

    private static func in_chksum(_ data: some Sequence<UInt8>) -> UInt16 {
        var iterator = data.makeIterator()
        var words = [UInt16]()

        while let byte = iterator.next() {
            let nextByte = iterator.next() ?? 0
            let word = UInt16(byte) << 8 | UInt16(nextByte)

            words.append(word)
        }

        let sum = words.reduce(0, &+)

        return ~sum
    }

    static func createICMPPacket(identifier: UInt16, sequenceNumber: UInt16) -> Data {
        var header = ICMPHeader(
            type: UInt8(ICMP_ECHO),
            code: 0,
            checksum: 0,
            identifier: identifier.bigEndian,
            sequenceNumber: sequenceNumber.bigEndian
        )
        header.checksum = withUnsafeBytes(of: &header) { in_chksum($0).bigEndian }

        return withUnsafeBytes(of: &header) { Data($0) }
    }

    static func parseICMPResponse(buffer: inout [UInt8], length: Int) throws -> ICMPHeader {
        try buffer.withUnsafeMutableBytes { bufferPointer in
            // Check IP packet size.
            guard length >= MemoryLayout<IPv4Header>.size else {
                throw Error.malformedResponse(.ipv4PacketTooSmall)
            }

            // Verify IPv4 header.
            let ipv4Header = bufferPointer.load(as: IPv4Header.self)
            let payloadLength = length - ipv4Header.headerLength

            guard payloadLength >= MemoryLayout<ICMPHeader>.size else {
                throw Error.malformedResponse(.icmpHeaderTooSmall)
            }

            guard ipv4Header.isIPv4Version else {
                throw Error.malformedResponse(.invalidIPVersion)
            }

            // Parse ICMP header.
            let icmpHeaderPointer = bufferPointer.baseAddress!
                .advanced(by: ipv4Header.headerLength)
                .assumingMemoryBound(to: ICMPHeader.self)

            // Copy server checksum.
            let serverChecksum = icmpHeaderPointer.pointee.checksum.bigEndian

            // Reset checksum field before calculating checksum.
            icmpHeaderPointer.pointee.checksum = 0

            // Verify ICMP checksum.
            let payloadPointer = UnsafeRawBufferPointer(
                start: icmpHeaderPointer,
                count: payloadLength
            )
            let clientChecksum = ICMP.in_chksum(payloadPointer)
            if clientChecksum != serverChecksum {
                throw Error.malformedResponse(.checksumMismatch(clientChecksum, serverChecksum))
            }

            // Ensure endianness before returning ICMP packet to delegate.
            var icmpHeader = icmpHeaderPointer.pointee
            icmpHeader.identifier = icmpHeader.identifier.bigEndian
            icmpHeader.sequenceNumber = icmpHeader.sequenceNumber.bigEndian
            icmpHeader.checksum = serverChecksum
            return icmpHeader
        }
    }
}

private extension IPv4Header {
    /// Returns IPv4 header length.
    var headerLength: Int {
        Int(versionAndHeaderLength & 0x0F) * MemoryLayout<UInt32>.size
    }

    /// Returns `true` if version header indicates IPv4.
    var isIPv4Version: Bool {
        (versionAndHeaderLength & 0xF0) == 0x40
    }
}
