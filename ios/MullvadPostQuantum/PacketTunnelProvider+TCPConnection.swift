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

/// Writes data to the in-tunnel TCP connection
///
/// This FFI function is called by Rust whenever there is data to be written to the in-tunnel TCP connection when exchanging
/// quantum-resistant pre shared keys.
///
/// Whenever the flow control is given back from the connection, acknowledge that data was written using `rawWriteAcknowledgement`.
/// - Parameters:
///   - rawConnection: A raw pointer to the in-tunnel TCP connection
///   - rawData: A raw pointer to the data to write in the connection
///   - dataLength: The length of data to write in the connection
///   - rawWriteAcknowledgement: An opaque pointer needed for write acknowledgement
@_cdecl("swift_nw_tcp_connection_send")
func tcpConnectionSend(
    rawConnection: UnsafeMutableRawPointer?,
    rawData: UnsafeMutableRawPointer,
    dataLength: UInt,
    rawWriteAcknowledgement: UnsafeMutableRawPointer?
) {
    guard let rawConnection, let rawWriteAcknowledgement else {
        handle_sent(0, rawWriteAcknowledgement)
        return
    }
    let tcpConnection = Unmanaged<NWTCPConnection>.fromOpaque(rawConnection).takeUnretainedValue()
    let data = Data(bytes: rawData, count: Int(dataLength))

    // The guarantee that all writes are sequential is done by virtue of not returning the execution context
    // to Rust before this closure is done executing.
    tcpConnection.write(data, completionHandler: { maybeError in
        if maybeError != nil {
            handle_sent(0, rawWriteAcknowledgement)
        } else {
            handle_sent(dataLength, rawWriteAcknowledgement)
        }
    })
}

/// Reads data to the in-tunnel TCP connection
///
/// This FFI function is called by Rust whenever there is data to be read from the in-tunnel TCP connection when exchanging
/// quantum-resistant pre shared keys.
///
/// Whenever the flow control is given back from the connection, acknowledge that data was read using `rawReadAcknowledgement`.
/// - Parameters:
///   - rawConnection: A raw pointer to the in-tunnel TCP connection
///   - rawReadAcknowledgement: An opaque pointer needed for read acknowledgement
@_cdecl("swift_nw_tcp_connection_read")
func tcpConnectionReceive(
    rawConnection: UnsafeMutableRawPointer?,
    rawReadAcknowledgement: UnsafeMutableRawPointer?
) {
    guard let rawConnection, let rawReadAcknowledgement else {
        handle_recv(nil, 0, rawReadAcknowledgement)
        return
    }
    let tcpConnection = Unmanaged<NWTCPConnection>.fromOpaque(rawConnection).takeUnretainedValue()
    tcpConnection.readMinimumLength(0, maximumLength: Int(UInt16.max)) { data, maybeError in
        if let data {
            if maybeError != nil {
                handle_recv(nil, 0, rawReadAcknowledgement)
            } else {
                handle_recv(data.map { $0 }, UInt(data.count), rawReadAcknowledgement)
            }
        }
    }
}

/// End sequence of a quantum-secure pre shared key exchange.
///
/// This FFI function is called by Rust when the quantum-secure pre shared key exchange has either failed, or succeeded.
/// When both the `rawPresharedKey` and the `rawEphemeralKey` are raw pointers to 32 bytes data arrays,
/// the quantum-secure key exchange is considered successful. In any other case, the exchange is considered failed.
///
/// - Parameters:
///   - rawPacketTunnel: A raw pointer to the running instance of `NEPacketTunnelProvider`
///   - rawPresharedKey: A raw pointer to the quantum-secure pre shared key
///   - rawEphemeralKey: A raw pointer to the ephemeral private key of the device
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
