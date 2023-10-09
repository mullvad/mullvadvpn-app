//
//  PingerProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

/// The result of processing ICMP reply.
public enum PingerReply {
    /// ICMP reply was successfully parsed.
    case success(_ sender: IPAddress, _ sequenceNumber: UInt16)

    /// ICMP reply couldn't be parsed.
    case parseError(Error)
}

/// The result of sending ICMP echo.
public struct PingerSendResult {
    /// Sequence id.
    public var sequenceNumber: UInt16

    /// How many bytes were sent.
    public var bytesSent: UInt
}

/// A type capable of sending and receving ICMP traffic.
public protocol PingerProtocol {
    var onReply: ((PingerReply) -> Void)? { get set }

    func openSocket(bindTo interfaceName: String?) throws
    func closeSocket()
    func send(to address: IPv4Address) throws -> PingerSendResult
}
