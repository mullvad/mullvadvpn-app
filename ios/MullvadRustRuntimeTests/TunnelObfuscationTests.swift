//
//  TunnelObfuscationTests.swift
//  MullvadRustRuntimeTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import Network
import WireGuardKitTypes
import XCTest

final class TunnelObfuscationTests: XCTestCase {
    /// A test public key generated from a random private key.
    private let testPublicKey = PrivateKey().publicKey

    override func setUp() {
        super.setUp()
        RustLogging.initialize()
    }

    func testRunningUdpOverTcpObfuscatorProxy() async throws {
        // Each packet is prefixed with u16 that contains a payload length.
        let preambleLength = MemoryLayout<UInt16>.size
        let markerData = Data([109, 117, 108, 108, 118, 97, 100])
        let packetLength = markerData.count + preambleLength

        let tcpListener = try UnsafeListener<TCPConnection>()
        try await tcpListener.start()

        let obfuscator = TunnelObfuscator(
            remoteAddress: IPv4Address.loopback,
            remotePort: tcpListener.listenPort,
            obfuscationProtocol: .udpOverTcp
        )
        obfuscator.start()

        // Accept incoming connections
        let connectionDataTask = Task {
            for await newConnection in tcpListener.newConnections {
                try await newConnection.start()

                return try await newConnection.receiveData(
                    minimumLength: packetLength,
                    maximumLength: packetLength
                )
            }
            throw POSIXError(.ECANCELED)
        }

        // Send marker data over UDP
        let connection = UDPConnection(remote: IPv4Address.loopback, port: obfuscator.localUdpPort)
        try await connection.start()
        try await connection.sendData(markerData)

        // Validate the sent data
        let receivedData = try await connectionDataTask.value
        XCTAssert(receivedData.count == packetLength)
        XCTAssertEqual(receivedData[preambleLength...], markerData)
    }

    func testRunningShadowsocksObfuscatorProxy() async throws {
        let markerData = Data([109, 117, 108, 108, 118, 97, 100])

        let localUdpListener = try UnsafeListener<UDPConnection>()
        try await localUdpListener.start()

        let localObfuscator = TunnelObfuscator(
            remoteAddress: IPv4Address.loopback,
            remotePort: localUdpListener.listenPort,
            obfuscationProtocol: .shadowsocks
        )
        localObfuscator.start()

        // Accept incoming connections
        let localConnectionDataTask = Task {
            for await obfuscatedConnection in localUdpListener.newConnections {
                try await obfuscatedConnection.start()

                let readDatagram = try await obfuscatedConnection.readSingleDatagram()
                // Write into the connection the unencrypted payload that was just read
                try await obfuscatedConnection.sendData(readDatagram)
                return readDatagram
            }
            throw POSIXError(.ECANCELED)
        }

        // Send marker data over UDP
        let connection = UDPConnection(remote: IPv4Address.loopback, port: localObfuscator.localUdpPort)
        try await connection.start()
        try await connection.sendData(markerData)
        let readDataFromObfuscator = try await connection.readSingleDatagram()

        // As the connection data is encrypted and this test does not run a shadowsocks server to decrypt the payload
        // The connection from the local UDP listener writes back what it read from the obfuscator, unencrypted
        _ = try await localConnectionDataTask.value
        XCTAssertEqual(readDataFromObfuscator, markerData)
    }

    /// Tests that the LWO obfuscator proxy can be started and stopped correctly via FFI.
    func testRunningLwoObfuscatorProxy() async throws {
        let markerData = Data([109, 117, 108, 108, 118, 97, 100])

        let localUdpListener = try UnsafeListener<UDPConnection>()
        try await localUdpListener.start()

        // Generate test keys for LWO
        let clientPrivateKey = PrivateKey()
        let serverPrivateKey = PrivateKey()

        let obfuscator = TunnelObfuscator(
            remoteAddress: IPv4Address.loopback,
            remotePort: localUdpListener.listenPort,
            obfuscationProtocol: .lwo(
                serverPublicKey: serverPrivateKey.publicKey,
                clientPublicKey: clientPrivateKey.publicKey
            )
        )
        obfuscator.start()

        // Verify the obfuscator has allocated a local UDP port
        XCTAssertNotEqual(obfuscator.localUdpPort, 0, "LWO obfuscator should allocate a local UDP port")

        // Accept incoming connections and echo back
        let connectionDataTask = Task {
            for await connection in localUdpListener.newConnections {
                try await connection.start()
                let readDatagram = try await connection.readSingleDatagram()
                try await connection.sendData(readDatagram)
                return readDatagram
            }
            throw POSIXError(.ECANCELED)
        }

        // Send marker data over UDP to the obfuscator's local port
        let connection = UDPConnection(remote: IPv4Address.loopback, port: obfuscator.localUdpPort)
        try await connection.start()
        try await connection.sendData(markerData)

        // Wait for the data to be received by the listener
        // The data will be obfuscated, so we just verify something was received
        let receivedData = try await connectionDataTask.value
        XCTAssertFalse(receivedData.isEmpty, "LWO obfuscator should forward data to the remote endpoint")

        // Stop the obfuscator - this tests that stop() doesn't crash or cause memory issues
        obfuscator.stop()
    }

    /// Tests that the LWO obfuscator can be started and stopped multiple times without memory issues.
    func testLwoObfuscatorStartStopMultipleTimes() async throws {
        let localUdpListener = try UnsafeListener<UDPConnection>()
        try await localUdpListener.start()

        let clientPrivateKey = PrivateKey()
        let serverPrivateKey = PrivateKey()

        // Create and destroy obfuscators multiple times to check for memory leaks/issues
        for iteration in 1...5 {
            let obfuscator = TunnelObfuscator(
                remoteAddress: IPv4Address.loopback,
                remotePort: localUdpListener.listenPort,
                obfuscationProtocol: .lwo(
                    serverPublicKey: serverPrivateKey.publicKey,
                    clientPublicKey: clientPrivateKey.publicKey
                )
            )

            obfuscator.start()
            XCTAssertNotEqual(
                obfuscator.localUdpPort, 0, "Iteration \(iteration): LWO obfuscator should allocate a port")

            obfuscator.stop()
        }
    }

    /// Tests that calling stop() on an already stopped obfuscator doesn't crash.
    func testLwoObfuscatorDoubleStopSafe() async throws {
        let localUdpListener = try UnsafeListener<UDPConnection>()
        try await localUdpListener.start()

        let clientPrivateKey = PrivateKey()
        let serverPrivateKey = PrivateKey()

        let obfuscator = TunnelObfuscator(
            remoteAddress: IPv4Address.loopback,
            remotePort: localUdpListener.listenPort,
            obfuscationProtocol: .lwo(
                serverPublicKey: serverPrivateKey.publicKey,
                clientPublicKey: clientPrivateKey.publicKey
            )
        )

        obfuscator.start()
        obfuscator.stop()
        // Second stop should be safe and not crash
        obfuscator.stop()
    }
}
