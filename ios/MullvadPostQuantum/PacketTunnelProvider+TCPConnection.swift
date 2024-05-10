//
//  PacketTunnelProvider+TCPConnection.swift
//  PacketTunnel
//
//  Created by Marco Nikic on 2024-02-15.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import NetworkExtension
import TalpidTunnelConfigClientProxy
import WireGuardKitTypes

@_cdecl("swift_nw_tcp_connection_send")
func tcpConnectionSend(
    connection: UnsafeMutableRawPointer?,
    data: UnsafeMutableRawPointer,
    dataLength: UInt,
    sender: UnsafeMutableRawPointer?
) {
    guard let connection, let sender else {
        handle_sent(0, sender)
        return
    }
    let tcpConnection = Unmanaged<NWTCPConnection>.fromOpaque(connection).takeUnretainedValue()
    let rawData = Data(bytes: data, count: Int(dataLength))

    // The guarantee that all writes are sequential is done by virtue of not returning the execution context
    // to Rust before this closure is done executing.
    tcpConnection.write(rawData, completionHandler: { maybeError in
        if maybeError != nil {
            handle_sent(0, sender)
        } else {
            handle_sent(dataLength, sender)
        }
    })
}

@_cdecl("swift_nw_tcp_connection_read")
func tcpConnectionReceive(
    connection: UnsafeMutableRawPointer?,
    sender: UnsafeMutableRawPointer?
) {
    guard let connection, let sender else {
        handle_recv(Data().map { $0 }, 0, sender)
        return
    }
    let tcpConnection = Unmanaged<NWTCPConnection>.fromOpaque(connection).takeUnretainedValue()
    tcpConnection.readMinimumLength(0, maximumLength: Int(UInt16.max)) { data, maybeError in
        if let data {
            if maybeError != nil {
                handle_recv(Data().map { $0 }, 0, sender)
            } else {
                handle_recv(data.map { $0 }, UInt(data.count), sender)
            }
        }
    }
}

@_cdecl("swift_post_quantum_key_ready")
func receivePostQuantumKey(
    rawPacketTunnel: UnsafeMutableRawPointer?,
    rawPresharedKey: UnsafeMutableRawPointer?,
    rawEphemeralKey: UnsafeMutableRawPointer?
) {
    guard
        let rawPacketTunnel,
        let postQuantumKeyReceiver = Unmanaged<NEPacketTunnelProvider>.fromOpaque(rawPacketTunnel)
        .takeUnretainedValue() as? PostQuantumKeyReceiving
    else { return }

    guard
        let rawPresharedKey,
        let rawEphemeralKey,
        let ephemeralKey = PrivateKey(rawValue: Data(bytes: rawEphemeralKey, count: 32)),
        let key = PreSharedKey(rawValue: Data(bytes: rawPresharedKey, count: 32))
    else {
        postQuantumKeyReceiver.keyExchangeFailed()
        return
    }

    postQuantumKeyReceiver.receivePostQuantumKey(key, ephemeralKey: ephemeralKey)
}
