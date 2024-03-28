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
    connection: UnsafeMutableRawPointer,
    data: UnsafeMutableRawPointer,
    dataLength: UInt,
    sender: UnsafeMutableRawPointer
) {
    let tcpConnection = Unmanaged<NWTCPConnection>.fromOpaque(connection).takeUnretainedValue()
    let rawData = Data(bytes: data, count: Int(dataLength))

    // The guarantee that no more than 2 writes happen in parallel is done by virtue of not returning the execution context
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
    connection: UnsafeMutableRawPointer,
    sender: UnsafeMutableRawPointer
) {
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
func receivePostQuantumKey(rawPacketTunnel: UnsafeMutableRawPointer, rawPresharedKey: UnsafeMutableRawPointer) {
    let packetTunnel = Unmanaged<NEPacketTunnelProvider>.fromOpaque(rawPacketTunnel).takeUnretainedValue()
    // TODO: The `rawPresharedKey` pointer might be null, this means the key exchanged failed, and we should try from the start again
    let presharedKey = Data(bytes: rawPresharedKey, count: 32)
    if let postQuantumKeyReceiver = packetTunnel as? PostQuantumKeyReceiving,
       let key = PreSharedKey(rawValue: presharedKey) {
        postQuantumKeyReceiver.receivePostQuantumKey(key)
    }
}
