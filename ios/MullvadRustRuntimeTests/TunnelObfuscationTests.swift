//
//  TunnelObfuscationTests.swift
//  MullvadRustRuntimeTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import Network
import XCTest

final class TunnelObfuscationTests: XCTestCase {
    func testRunningUdpOverTcpObfuscatorProxy() async throws {
        // Each packet is prefixed with u16 that contains a payload length.
        let preambleLength = MemoryLayout<UInt16>.size
        let markerData = Data([109, 117, 108, 108, 118, 97, 100])
        let packetLength = markerData.count + preambleLength

        let tcpListener = try UnsafeListener<TCPConnection>()
        try await tcpListener.start()

        let obfuscator = TunnelObfuscator(
            remoteAddress: IPv4Address.loopback,
            tcpPort: tcpListener.listenPort,
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
            tcpPort: localUdpListener.listenPort,
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
}
