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

        let tcpListener = try TCPListener()
        try await tcpListener.start()

        let obfuscator = TunnelObfuscator(remoteAddress: IPv4Address.loopback, tcpPort: tcpListener.listenPort)
        obfuscator.start()

        let expectation = expectation(description: "Should send packet over UDP and receive over TCP.")

        // Accept incoming connections
        let acceptConnectionsTask = Task.detached {
            for await newConnection in tcpListener.newConnections {
                try await newConnection.start()

                let receivedData = try await newConnection.receiveData(
                    minimumLength: packetLength,
                    maximumLength: packetLength
                )

                XCTAssert(receivedData.count == packetLength)
                XCTAssertEqual(receivedData[preambleLength...], markerData)

                expectation.fulfill()

                // Break the loop to exit task.
                break
            }
        }

        // Send marker data over UDP
        let sendDataTask = Task.detached {
            let connection = UDPConnection(remote: IPv4Address.loopback, port: obfuscator.localUdpPort)
            try await connection.start()
            try await connection.sendData(markerData)
        }

        // Timeout if nothing happened within reasonable time interval.
        Task.detached {
            await self.fulfillment(of: [expectation], timeout: 2)
        }

        try await sendDataTask.value
        try await acceptConnectionsTask.value
    }
}
