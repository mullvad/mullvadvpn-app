//
//  TunnelPinger.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-07-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import Network
import PacketTunnelCore
import WireGuardKit

public final class TunnelPinger: PingerProtocol {
    private var sequenceNumber: UInt16 = 0
    private let stateLock = NSRecursiveLock()
    private let pingReceiveQueue: DispatchQueue
    private let replyQueue: DispatchQueue
    private var destAddress: IPv4Address?
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

    private var pingProvider: ICMPPingProvider

    private let logger: Logger

    init(pingProvider: ICMPPingProvider, replyQueue: DispatchQueue) {
        self.pingProvider = pingProvider
        self.replyQueue = replyQueue
        self.pingReceiveQueue = DispatchQueue(label: "PacketTunnel.Receive.icmp")
        self.logger = Logger(label: "TunnelPinger")
    }

    deinit {
        pingProvider.closeICMP()
    }

    public func openSocket(bindTo interfaceName: String?, destAddress: IPv4Address) throws {
        try pingProvider.openICMP(address: destAddress)
        self.destAddress = destAddress
        pingReceiveQueue.async { [weak self] in
            while let self {
                do {
                    let seq = try pingProvider.receiveICMP()

                    replyQueue.async { [weak self] in
                        self?.onReply?(PingerReply.success(destAddress, UInt16(seq)))
                    }
                } catch {
                    replyQueue.async { [weak self] in
                        self?.onReply?(PingerReply.parseError(error))
                    }
                    return
                }
            }
        }
    }

    public func closeSocket() {
        pingProvider.closeICMP()
        self.destAddress = nil
    }

    public func send() throws -> PingerSendResult {
        let sequenceNumber = nextSequenceNumber()

        guard let destAddress else { throw WireGuardAdapterError.invalidState }
        // NOTE: we cheat here by returning the destination address we were passed, rather than parsing it from the packet on the other side of the FFI boundary.
        try pingProvider.sendICMPPing(seqNumber: sequenceNumber)


        return PingerSendResult(sequenceNumber: UInt16(sequenceNumber))
    }

    private func nextSequenceNumber() -> UInt16 {
        stateLock.lock()
        let (nextValue, _) = sequenceNumber.addingReportingOverflow(1)
        sequenceNumber = nextValue
        stateLock.unlock()

        return nextValue
    }
}
