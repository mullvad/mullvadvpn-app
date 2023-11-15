//
//  TunnelObfuscationTests.swift
//  TunnelObfuscationTests
//
//  Created by pronebird on 27/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Network
import TunnelObfuscation
import XCTest

final class TunnelObfuscationTests: XCTestCase {
    func testRunningObfuscatorProxy() async throws {
        // Each packet is prefixed with u16 that contains a payload length.
        let preambleLength = MemoryLayout<UInt16>.size
        let markerData = Data([109, 117, 108, 108, 118, 97, 100])
        let packetLength = markerData.count + preambleLength

        let tcpListener = try TCPUnsafeListener()
        try await tcpListener.start()

        let obfuscator = UDPOverTCPObfuscator(remoteAddress: IPv4Address.loopback, tcpPort: tcpListener.listenPort)
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
}
