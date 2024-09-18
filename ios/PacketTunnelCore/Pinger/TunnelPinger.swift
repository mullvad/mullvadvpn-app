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
    private let pingQueue: DispatchQueue
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
        self.pingQueue = DispatchQueue(label: "PacketTunnel.icmp")
        self.logger = Logger(label: "TunnelPinger")
    }

    deinit {
        pingProvider.closeICMP()
    }

    public func openSocket(bindTo interfaceName: String?, destAddress: IPv4Address) throws {
        try pingProvider.openICMP(address: destAddress)
        self.destAddress = destAddress
    }

    public func closeSocket() {
        pingProvider.closeICMP()
        self.destAddress = nil
    }

    public func send() throws -> PingerSendResult {
        let sequenceNumber = nextSequenceNumber()
        logger.debug("*** sending ping \(sequenceNumber)")

        pingQueue.async { [weak self] in
            guard let self, let destAddress else { return }
            let reply: PingerReply
            do {
                try pingProvider.sendICMPPing(seqNumber: sequenceNumber)
                // NOTE: we cheat here by returning the destination address we were passed, rather than parsing it from the packet on the other side of the FFI boundary.
                reply = .success(destAddress, sequenceNumber)
            } catch {
                reply = .parseError(error)
            }
            self.logger.debug("--- Pinger reply: \(reply)")

            replyQueue.async { [weak self] in
                guard let self else { return }
                self.onReply?(reply)
            }
        }

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
