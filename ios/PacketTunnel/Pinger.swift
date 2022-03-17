//
//  Pinger.swift
//  PacketTunnel
//
//  Created by pronebird on 21/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct Network.IPv4Address
import Logging

final class Pinger {
    // Sender identifier passed along with ICMP packet.
    private let identifier: UInt16 = 757

    private var sequenceNumber: UInt16 = 0
    private var socket: CFSocket?

    private let address: IPv4Address
    private let interfaceName: String?

    private let logger = Logger(label: "Pinger")
    private let stateLock = NSRecursiveLock()
    private var timer: DispatchSourceTimer?

    init(address: IPv4Address, interfaceName: String?) {
        self.address = address
        self.interfaceName = interfaceName
    }

    deinit {
        stop()
    }

    func start(delay: DispatchTimeInterval, repeating repeatInterval: DispatchTimeInterval) -> Result<(), Pinger.Error> {
        stateLock.lock()
        defer { stateLock.unlock() }

        stop()

        guard let newSocket = CFSocketCreate(kCFAllocatorDefault, AF_INET, SOCK_DGRAM, IPPROTO_ICMP, 0, nil, nil) else {
            return .failure(.createSocket)
        }

        let flags = CFSocketGetSocketFlags(newSocket)
        if (flags & kCFSocketCloseOnInvalidate) == 0 {
            CFSocketSetSocketFlags(newSocket, flags | kCFSocketCloseOnInvalidate)
        }

        if case .failure(let error) = bindSocket(newSocket) {
            return .failure(error)
        }

        guard let runLoop = CFSocketCreateRunLoopSource(kCFAllocatorDefault, newSocket, 0) else {
            return .failure(.createRunLoop)
        }

        CFRunLoopAddSource(CFRunLoopGetMain(), runLoop, .defaultMode)

        let newTimer = DispatchSource.makeTimerSource()
        newTimer.setEventHandler { [weak self] in
            self?.send()
        }

        socket = newSocket
        timer = newTimer

        newTimer.schedule(wallDeadline: .now() + delay, repeating: repeatInterval)
        newTimer.resume()

        return .success(())
    }

    func stop() {
        stateLock.lock()
        defer { stateLock.unlock() }

        if let socket = socket {
            CFSocketInvalidate(socket)
        }

        socket = nil

        timer?.cancel()
        timer = nil
    }

    private func send() {
        stateLock.lock()
        guard let socket = socket else {
            stateLock.unlock()
            return
        }
        stateLock.unlock()

        var sa = sockaddr_in()
        sa.sin_len = UInt8(MemoryLayout.size(ofValue: sa))
        sa.sin_family = sa_family_t(AF_INET)
        sa.sin_addr = address.rawValue.withUnsafeBytes { buffer in
            return buffer.bindMemory(to: in_addr.self).baseAddress!.pointee
        }

        let sequenceNumber = nextSequenceNumber()
        let packetData = Self.createICMPPacket(identifier: identifier, sequenceNumber: sequenceNumber, payload: nil)

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

        if bytesSent == -1 {
            logger.debug("Failed to send echo (errno: \(errno)).")
        }
    }

    private func nextSequenceNumber() -> UInt16 {
        stateLock.lock()
        let (partialValue, isOverflow) = sequenceNumber.addingReportingOverflow(1)
        let nextSequenceNumber = isOverflow ? 0 : partialValue

        sequenceNumber = nextSequenceNumber
        stateLock.unlock()

        return nextSequenceNumber
    }

    private func bindSocket(_ socket: CFSocket) -> Result<(), Pinger.Error> {
        guard let interfaceName = interfaceName else {
            logger.debug("Interface is not specified.")
            return .success(())
        }

        var index = if_nametoindex(interfaceName)
        guard index > 0 else {
            return .failure(.mapInterfaceNameToIndex(errno))
        }

        logger.debug("Bind socket to \"\(interfaceName)\" (index: \(index))...")

        let result = setsockopt(
            CFSocketGetNative(socket),
            IPPROTO_IP,
            IP_BOUND_IF,
            &index,
            socklen_t(MemoryLayout.size(ofValue: index))
        )

        if result == -1 {
            logger.error("Failed to bind socket to \"\(interfaceName)\" (index: \(index), errno: \(errno)).")

            return .failure(.bindSocket(errno))
        } else {
            return .success(())
        }
    }

    private class func createICMPPacket(identifier: UInt16, sequenceNumber: UInt16, payload: Data?) -> Data {
        // Create data buffer.
        var data = Data()

        // ICMP type.
        data.append(UInt8(ICMP_ECHO))

        // Code.
        data.append(UInt8(0))

        // Checksum.
        withUnsafeBytes(of: UInt16(0)) { data.append(Data($0)) }

        // Identifier.
        withUnsafeBytes(of: identifier.bigEndian) { data.append(Data($0)) }

        // Sequence number.
        withUnsafeBytes(of: sequenceNumber.bigEndian) { data.append(Data($0)) }

        // Append payload.
        if let payload = payload {
            data.append(contentsOf: payload)
        }

        // Calculate checksum.
        let checksum = in_chksum(data)

        // Inject computed checksum into the packet.
        data.withUnsafeMutableBytes { buffer in
            buffer.storeBytes(of: checksum, toByteOffset: 2, as: UInt16.self)
        }

        return data
    }
}

extension Pinger {
    enum Error: LocalizedError, Equatable {
        /// Failure to create a socket.
        case createSocket

        /// Failure to map interface name to index.
        case mapInterfaceNameToIndex(Int32)

        /// Failure to bind socket to interface.
        case bindSocket(Int32)

        /// Failure to create a runloop for socket.
        case createRunLoop

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
            }
        }
    }
}

private func in_chksum(_ data: Data) -> UInt16 {
    return data.withUnsafeBytes { buffer in
        let length = buffer.count

        var sum: Int32 = 0

        let isOdd = length  % 2 != 0
        let strideTo = isOdd ? length - 1 : length

        for offset in stride(from: 0, to: strideTo, by: 2) {
            let word = buffer.load(fromByteOffset: offset, as: UInt16.self)
            sum += Int32(word)
        }

        if isOdd {
            let byte = buffer.load(fromByteOffset: length - 1, as: UInt8.self)
            sum += Int32(byte)
        }

        sum = (sum >> 16) + (sum & 0xffff)
        sum += (sum >> 16)

        return UInt16(truncatingIfNeeded: ~sum)
    }
}
