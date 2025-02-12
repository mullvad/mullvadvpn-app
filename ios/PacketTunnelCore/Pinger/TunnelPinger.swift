//
//  TunnelPinger.swift
//  PacketTunnelCore
//
//  Created by Andrew Bulhak on 2024-07-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import Network
import PacketTunnelCore
import WireGuardKit

public final class TunnelPinger: PingerProtocol {
    private var sequenceNumber: UInt16 = 0
    private let stateLock = NSLock()
    private let pingReceiveQueue: DispatchQueue
    private let replyQueue: DispatchQueue
    private var destAddress: IPv4Address?
    /// Always accessed from the `replyQueue` and is assigned once, on the main thread of the PacketTunnel. It is thread safe.
    public var onReply: (@Sendable (PingerReply) -> Void)?
    private var pingProvider: ICMPPingProvider

    nonisolated(unsafe) private let logger: Logger

    init(pingProvider: ICMPPingProvider, replyQueue: DispatchQueue) {
        self.pingProvider = pingProvider
        self.replyQueue = replyQueue
        self.pingReceiveQueue = DispatchQueue(label: "PacketTunnel.Receive.icmp")
        self.logger = Logger(label: "TunnelPinger")
    }

    public func startPinging(destAddress: IPv4Address) {
        stateLock.withLock {
            self.destAddress = destAddress
        }
        pingReceiveQueue.async { [weak self] in
            while let self {
                do {
                    let seq = try pingProvider.receiveICMP()
                    logger.debug("received seq \(seq)")

                    replyQueue.async { [weak self] in
                        self?.stateLock.withLock {
                            self?.onReply?(PingerReply.success(destAddress, UInt16(seq)))
                        }
                    }
                } catch {
                    replyQueue.async { [weak self] in
                        self?.stateLock.withLock {
                            if self?.destAddress != nil {
                                self?.onReply?(PingerReply.parseError(error))
                            }
                        }
                    }
                    return
                }
            }
        }
    }

    public func stopPinging() {
        stateLock.withLock {
            self.destAddress = nil
        }
    }

    public func send() throws -> PingerSendResult {
        let sequenceNumber = nextSequenceNumber()

        stateLock.lock()
        defer { stateLock.unlock() }
        guard destAddress != nil else { throw WireGuardAdapterError.invalidState }
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
