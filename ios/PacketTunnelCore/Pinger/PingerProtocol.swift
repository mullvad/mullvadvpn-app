//
//  PingerProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

public enum PingerReply {
    case success(_ sender: IPAddress, _ sequenceNumber: UInt16)
    case parseError(Error)
}

public struct PingerSendResult {
    public var sequenceNumber: UInt16
    public var bytesSent: UInt16
}

public protocol PingerProtocol {
    var onReply: ((PingerReply) -> Void)? { get set }

    func openSocket(bindTo interfaceName: String?) throws
    func closeSocket()
    func send(to address: IPv4Address) throws -> PingerSendResult
}
