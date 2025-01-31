//
//  PingerProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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

    public init(sequenceNumber: UInt16) {
        self.sequenceNumber = sequenceNumber
    }
}

/// A type capable of sending and receving ICMP traffic.
public protocol PingerProtocol: AnyObject, Sendable {
    var onReply: (@Sendable (PingerReply) -> Void)? { get set }

    func startPinging(destAddress: IPv4Address)
    func stopPinging()
    func send() throws -> PingerSendResult
}
