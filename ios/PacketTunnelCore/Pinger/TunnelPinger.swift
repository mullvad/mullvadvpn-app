//
//  TunnelPinger.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-07-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

public final class TunnelPinger: PingerProtocol {
    private let stateLock = NSRecursiveLock()
    private var _onReply: ((PingerReply) -> Void)?
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
    // Sender identifier passed along with ICMP packet.
    private let identifier: UInt16
    
    public func openSocket(bindTo interfaceName: String?) throws {
        <#code#>
    }
    
    public func closeSocket() {
        <#code#>
    }
    
    public func send(to address: IPv4Address) throws -> PingerSendResult {
        <#code#>
    }
    
    
}
