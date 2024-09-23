//
//  Pinger.swift
//  PacketTunnelCore
//
//  Created by pronebird on 21/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

// This is the legacy Pinger using native TCP/IP networking.

/// ICMP client.
public final class Pinger: PingerProtocol, @unchecked Sendable {
    // Socket read buffer size.
    private static let bufferSize = 65535

    // Sender identifier passed along with ICMP packet.
    private let identifier: UInt16

    private var sequenceNumber: UInt16 = 0
    private var socket: CFSocket?
    private var readBuffer = [UInt8](repeating: 0, count: bufferSize)
    private let stateLock = NSRecursiveLock()
    private let replyQueue: DispatchQueue
    private var destAddress: IPv4Address?

    public var onReply: ((PingerReply) -> Void)? {
        get {
            stateLock.withLock {
                return _onReply
            }
        }
        set {
            stateLock.withLock {
                _onReply = newValue
            }
        }
    }

    private var _onReply: ((PingerReply) -> Void)?

    deinit {
        closeSocket()
    }

    public init(identifier: UInt16 = 757, replyQueue: DispatchQueue) {
        self.identifier = identifier
        self.replyQueue = replyQueue
    }

    /// Open socket and optionally bind it to the given interface.
    /// Automatically closes the previously opened socket when called multiple times in a row.
    public func openSocket(bindTo interfaceName: String?, destAddress: IPv4Address) throws {
        stateLock.lock()
        defer { stateLock.unlock() }

        closeSocket()

        self.destAddress = destAddress

        var context = CFSocketContext()
        context.info = Unmanaged.passUnretained(self).toOpaque()

        guard let newSocket = CFSocketCreate(
            kCFAllocatorDefault,
            AF_INET,
            SOCK_DGRAM,
            IPPROTO_ICMP,
            CFSocketCallBackType.readCallBack.rawValue,
            { socket, callbackType, _, _, info in
                guard let socket, let info, callbackType == .readCallBack else {
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

        if let interfaceName {
            try bindSocket(newSocket, to: interfaceName)
        }

        guard let runLoop = CFSocketCreateRunLoopSource(kCFAllocatorDefault, newSocket, 0) else {
            throw Error.createRunLoop
        }

        CFRunLoopAddSource(CFRunLoopGetMain(), runLoop, .commonModes)

        socket = newSocket
    }

    public func closeSocket() {
        stateLock.lock()
        defer { stateLock.unlock() }

        if let socket {
            CFSocketInvalidate(socket)

            self.socket = nil
        }
    }

    /// Send ping packet to the given address.
    /// Returns `PingerSendResult` on success, otherwise throws a `Pinger.Error`.
    public func send() throws -> PingerSendResult {
        stateLock.lock()
        defer { stateLock.unlock() }

        guard let socket else {
            throw Error.closedSocket
        }

        guard let destAddress else {
            throw Error.parseIPAddress
        }

        var sa = sockaddr_in()
        sa.sin_len = UInt8(MemoryLayout.size(ofValue: sa))
        sa.sin_family = sa_family_t(AF_INET)
        sa.sin_addr = destAddress.rawValue.withUnsafeBytes { buffer in
            buffer.bindMemory(to: in_addr.self).baseAddress!.pointee
        }

        let sequenceNumber = nextSequenceNumber()
        let packetData = ICMP.createICMPPacket(
            identifier: identifier,
            sequenceNumber: sequenceNumber
        )

        let bytesSent = packetData.withUnsafeBytes { dataBuffer -> Int in
            withUnsafeBytes(of: &sa) { bufferPointer in
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

        return PingerSendResult(sequenceNumber: sequenceNumber)
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

            let icmpHeader = try ICMP.parseICMPResponse(buffer: &readBuffer, length: bytesRead)
            guard icmpHeader.identifier == identifier else {
                throw Error.clientIdentifierMismatch
            }
            guard icmpHeader.type == ICMP_ECHOREPLY else {
                throw Error.invalidICMPType(icmpHeader.type)
            }
            guard let sender = Self.makeIPAddress(from: address) else { throw Error.parseIPAddress }

            replyQueue.async {
                self.onReply?(.success(sender, icmpHeader.sequenceNumber))
            }
        } catch Pinger.Error.clientIdentifierMismatch {
            // Ignore responses from other senders.
        } catch {
            replyQueue.async {
                self.onReply?(.parseError(error))
            }
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

    private static func makeIPAddress(from sa: sockaddr) -> IPAddress? {
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
                buffer.bindMemory(to: sockaddr_in6.self).baseAddress
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
    public enum Error: LocalizedError {
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

        /// Unexpected ICMP reply type
        case invalidICMPType(UInt8)

        /// Response identifier does not match the sender identifier.
        case clientIdentifierMismatch

        /// Failure to parse IP address.
        case parseIPAddress

        public var errorDescription: String? {
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
            case let .invalidICMPType(type):
                return "Unexpected ICMP reply type: \(type)"
            case .clientIdentifierMismatch:
                return "Response identifier does not match the sender identifier."
            case .parseIPAddress:
                return "Failed to parse IP address."
            }
        }
    }
}
