//
//  Pinger.swift
//  PacketTunnel
//
//  Created by pronebird on 21/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import protocol Network.IPAddress
import struct Network.IPv4Address
import struct Network.IPv6Address

protocol PingerDelegate: AnyObject {
    func pinger(
        _ pinger: Pinger,
        didReceiveResponseFromSender senderAddress: IPAddress,
        icmpHeader: ICMPHeader
    )

    func pinger(
        _ pinger: Pinger,
        didFailWithError error: Error
    )
}

final class Pinger {
    struct SendResult {
        var sequenceNumber: UInt16
        var bytesSent: UInt16
    }

    // Socket read buffer size.
    private static let bufferSize = 65535

    // Sender identifier passed along with ICMP packet.
    private let identifier: UInt16

    private var sequenceNumber: UInt16 = 0
    private var socket: CFSocket?
    private var readBuffer = [UInt8](repeating: 0, count: bufferSize)
    private let stateLock = NSRecursiveLock()

    private weak var _delegate: PingerDelegate?
    private let delegateQueue: DispatchQueue

    var delegate: PingerDelegate? {
        get {
            stateLock.lock()
            defer { stateLock.unlock() }

            return _delegate
        }
        set {
            stateLock.lock()
            defer { stateLock.unlock() }

            _delegate = newValue
        }
    }

    deinit {
        closeSocket()
    }

    init(identifier: UInt16 = 757, delegateQueue: DispatchQueue) {
        self.identifier = identifier
        self.delegateQueue = delegateQueue
    }

    /// Open socket and optionally bind it to the given interface.
    /// Automatically closes the previously opened socket when called multiple times in a row.
    func openSocket(bindTo interfaceName: String?) throws {
        stateLock.lock()
        defer { stateLock.unlock() }

        closeSocket()

        var context = CFSocketContext()
        context.info = Unmanaged.passUnretained(self).toOpaque()

        guard let newSocket = CFSocketCreate(
            kCFAllocatorDefault,
            AF_INET,
            SOCK_DGRAM,
            IPPROTO_ICMP,
            CFSocketCallBackType.readCallBack.rawValue,
            { socket, callbackType, address, data, info in
                guard let socket = socket, let info = info, callbackType == .readCallBack else {
                    return
                }

                let pinger = Unmanaged<Pinger>.fromOpaque(info).takeUnretainedValue()

                pinger.readSocket(socket)
            },
            &context
        ) else {
            throw Error.createSocket
        }

        let flags = CFSocketGetSocketFlags(newSocket)
        if (flags & kCFSocketCloseOnInvalidate) == 0 {
            CFSocketSetSocketFlags(newSocket, flags | kCFSocketCloseOnInvalidate)
        }

        if let interfaceName = interfaceName {
            try bindSocket(newSocket, to: interfaceName)
        }

        guard let runLoop = CFSocketCreateRunLoopSource(kCFAllocatorDefault, newSocket, 0) else {
            throw Error.createRunLoop
        }

        CFRunLoopAddSource(CFRunLoopGetMain(), runLoop, .commonModes)

        socket = newSocket
    }

    func closeSocket() {
        stateLock.lock()
        defer { stateLock.unlock() }

        if let socket = socket {
            CFSocketInvalidate(socket)

            self.socket = nil
        }
    }

    /// Send ping packet to the given address.
    /// Returns `SendResult` on success, otherwise throws a `Pinger.Error`.
    func send(to address: IPv4Address) throws -> SendResult {
        stateLock.lock()
        guard let socket = socket else {
            stateLock.unlock()
            throw Error.closedSocket
        }
        stateLock.unlock()

        var sa = sockaddr_in()
        sa.sin_len = UInt8(MemoryLayout.size(ofValue: sa))
        sa.sin_family = sa_family_t(AF_INET)
        sa.sin_addr = address.rawValue.withUnsafeBytes { buffer in
            return buffer.bindMemory(to: in_addr.self).baseAddress!.pointee
        }

        let sequenceNumber = nextSequenceNumber()
        let packetData = Self.createICMPPacket(
            identifier: identifier,
            sequenceNumber: sequenceNumber
        )

        let bytesSent = packetData.withUnsafeBytes { dataBuffer -> Int in
            return withUnsafeBytes(of: &sa) { bufferPointer in
                let sockaddrPointer = bufferPointer.bindMemory(to: sockaddr.self).baseAddress!

                return sendto(
                    CFSocketGetNative(socket),
                    dataBuffer.baseAddress!,
                    dataBuffer.count,
                    0,
                    sockaddrPointer,
                    socklen_t(MemoryLayout<sockaddr_in>.size)
                )
            }
        }

        guard bytesSent >= 0 else {
            throw Error.sendPacket(errno)
        }

        return SendResult(sequenceNumber: sequenceNumber, bytesSent: UInt16(bytesSent))
    }

    private func nextSequenceNumber() -> UInt16 {
        stateLock.lock()
        let (nextValue, _) = sequenceNumber.addingReportingOverflow(1)
        sequenceNumber = nextValue
        stateLock.unlock()

        return nextValue
    }

    private func readSocket(_ socket: CFSocket) {
        var address = sockaddr()
        var addressLength = socklen_t(MemoryLayout.size(ofValue: address))

        let bytesRead = recvfrom(
            CFSocketGetNative(socket),
            &readBuffer,
            Self.bufferSize,
            0,
            &address,
            &addressLength
        )

        do {
            guard bytesRead > 0 else { throw Error.receivePacket(errno) }

            let icmpHeader = try parseICMPResponse(buffer: &readBuffer, length: bytesRead)
            guard let sender = Self.makeIPAddress(from: address) else { throw Error.parseIPAddress }

            delegateQueue.async {
                self.delegate?.pinger(
                    self,
                    didReceiveResponseFromSender: sender,
                    icmpHeader: icmpHeader
                )
            }
        } catch Pinger.Error.clientIdentifierMismatch {
            // Ignore responses from other senders.
        } catch {
            delegateQueue.async {
                self.delegate?.pinger(self, didFailWithError: error)
            }
        }
    }

    private func parseICMPResponse(buffer: inout [UInt8], length: Int) throws -> ICMPHeader {
        return try buffer.withUnsafeMutableBytes { bufferPointer in
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

            // Check if ICMP response identifier matches the one from sender.
            guard icmpHeaderPointer.pointee.identifier.bigEndian == identifier else {
                throw Error.clientIdentifierMismatch
            }

            // Verify ICMP type.
            guard icmpHeaderPointer.pointee.type == ICMP_ECHOREPLY else {
                throw Error.malformedResponse(.invalidEchoReplyType)
            }

            // Copy server checksum.
            let serverChecksum = icmpHeaderPointer.pointee.checksum.bigEndian

            // Reset checksum field before calculating checksum.
            icmpHeaderPointer.pointee.checksum = 0

            // Verify ICMP checksum.
            let payloadPointer = UnsafeRawBufferPointer(
                start: icmpHeaderPointer,
                count: payloadLength
            )
            let clientChecksum = in_chksum(payloadPointer)
            if clientChecksum != serverChecksum {
                throw Error.malformedResponse(.checksumMismatch(clientChecksum, serverChecksum))
            }

            // Ensure endianess before returning ICMP packet to delegate.
            var icmpHeader = icmpHeaderPointer.pointee
            icmpHeader.identifier = icmpHeader.identifier.bigEndian
            icmpHeader.sequenceNumber = icmpHeader.sequenceNumber.bigEndian
            icmpHeader.checksum = serverChecksum
            return icmpHeader
        }
    }

    private func bindSocket(_ socket: CFSocket, to interfaceName: String) throws {
        var index = if_nametoindex(interfaceName)
        guard index > 0 else {
            throw Error.mapInterfaceNameToIndex(errno)
        }

        let result = setsockopt(
            CFSocketGetNative(socket),
            IPPROTO_IP,
            IP_BOUND_IF,
            &index,
            socklen_t(MemoryLayout.size(ofValue: index))
        )

        if result == -1 {
            throw Error.bindSocket(errno)
        }
    }

    private class func createICMPPacket(identifier: UInt16, sequenceNumber: UInt16) -> Data {
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

    private class func makeIPAddress(from sa: sockaddr) -> IPAddress? {
        if sa.sa_family == AF_INET {
            return withUnsafeBytes(of: sa) { buffer -> IPAddress? in
                buffer.bindMemory(to: sockaddr_in.self).baseAddress.flatMap { boundPointer in
                    var saddr = boundPointer.pointee
                    let data = Data(bytes: &saddr.sin_addr, count: MemoryLayout<in_addr>.size)

                    return IPv4Address(data, nil)
                }
            }
        }

        if sa.sa_family == AF_INET6 {
            return withUnsafeBytes(of: sa) { buffer in
                return buffer.bindMemory(to: sockaddr_in6.self).baseAddress
                    .flatMap { boundPointer in
                        var saddr6 = boundPointer.pointee
                        let data = Data(
                            bytes: &saddr6.sin6_addr,
                            count: MemoryLayout<in6_addr>.size
                        )

                        return IPv6Address(data)
                    }
            }
        }

        return nil
    }
}

extension Pinger {
    enum Error: LocalizedError {
        /// Failure to create a socket.
        case createSocket

        /// Failure to map interface name to index.
        case mapInterfaceNameToIndex(Int32)

        /// Failure to bind socket to interface.
        case bindSocket(Int32)

        /// Failure to create a runloop for socket.
        case createRunLoop

        /// Failure to send a packet due to socket being closed.
        case closedSocket

        /// Failure to send packet. Contains the `errno`.
        case sendPacket(Int32)

        /// Failure to receive packet. Contains the `errno`.
        case receivePacket(Int32)

        /// Response identifier does not match the sender identifier.
        case clientIdentifierMismatch

        /// Malformed response.
        case malformedResponse(MalformedResponseReason)

        /// Failure to parse IP address.
        case parseIPAddress

        var errorDescription: String? {
            switch self {
            case .createSocket:
                return "Failure to create socket."
            case .mapInterfaceNameToIndex:
                return "Failure to map interface name to index."
            case .bindSocket:
                return "Failure to bind socket to interface."
            case .createRunLoop:
                return "Failure to create run loop for socket."
            case .closedSocket:
                return "Socket is closed."
            case let .sendPacket(code):
                return "Failure to send packet (errno: \(code))."
            case let .receivePacket(code):
                return "Failure to receive packet (errno: \(code))."
            case .clientIdentifierMismatch:
                return "Response identifier does not match the sender identifier."
            case let .malformedResponse(reason):
                return "Malformed response: \(reason)."
            case .parseIPAddress:
                return "Failed to parse IP address."
            }
        }
    }

    enum MalformedResponseReason {
        case ipv4PacketTooSmall
        case icmpHeaderTooSmall
        case invalidIPVersion
        case invalidEchoReplyType
        case checksumMismatch(UInt16, UInt16)
    }
}

private func in_chksum<S>(_ data: S) -> UInt16 where S: Sequence, S.Element == UInt8 {
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

private extension IPv4Header {
    /// Returns IPv4 header length.
    var headerLength: Int {
        return Int(versionAndHeaderLength & 0x0F) * MemoryLayout<UInt32>.size
    }

    /// Returns `true` if version header indicates IPv4.
    var isIPv4Version: Bool {
        return (versionAndHeaderLength & 0xF0) == 0x40
    }
}
