// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
public protocol PingerProtocol: Sendable {
    var onReply: ((PingerReply) -> Void)? { get set }

    func startPinging(destAddress: IPv4Address) throws
    func stopPinging()
    func send() throws -> PingerSendResult
}
