//
//  TrafficGenerator.swift
//  MullvadVPNUITests
//
//  Created by Niklas Berglund on 2024-06-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Network
import XCTest

class TrafficGenerator {
    let destinationHost: String
    let port: Int
    var connection: NWConnection
    let dispatchQueue = DispatchQueue(label: "TrafficGeneratorDispatchQueue", qos: .unspecified)
    var sendDataTimer: DispatchSourceTimer

    init(destinationHost: String, port: Int) {
        self.destinationHost = destinationHost
        self.port = port

        sendDataTimer = DispatchSource.makeTimerSource(queue: dispatchQueue)
        let params = NWParameters.tcp
//        params.requiredInterfaceType = .other
        connection = NWConnection(
            host: NWEndpoint.Host(destinationHost),
            port: NWEndpoint.Port(integerLiteral: UInt16(port)),
            using: params
        )
        setupConnection()
    }

    func reconnect() {
        print("Attempting to reconnect")
        self.connection.forceCancel()

        connection = recreateConnection()
        self.setupConnection()
    }

    func recreateConnection() -> NWConnection {
        let params = NWParameters.tcp
//        params.requiredInterfaceType = .other
        return NWConnection(
            host: NWEndpoint.Host(destinationHost),
            port: NWEndpoint.Port(integerLiteral: UInt16(port)),
            using: params
        )
    }


    func setupConnection() {
        print("Setting up connection...")
        let doneAttemptingConnectExpecation = XCTestExpectation(description: "Done attemping to connect")

        connection.stateUpdateHandler = { state in
            switch state {
            case .ready:
                print("Ready")
                self.sendDataTimer.resume()
                doneAttemptingConnectExpecation.fulfill()
            case let .failed(error):
                print("Failed to connect: \(error)")
                self.sendDataTimer.cancel()
                self.reconnect()
            case .preparing:
                print("Preparing connection...")
            case .setup:
                print("Setting upp connection...")
            case let .waiting(error):
                print("Waiting to connect: \(error)")
            case .cancelled:
                self.sendDataTimer.suspend()
                print("Cancelled connection")
                self.reconnect()
            default:
                break
            }
        }
        
        connection.start(queue: dispatchQueue)

        XCTWaiter().wait(for: [doneAttemptingConnectExpecation], timeout: 10.0)
    }

    public func startGeneratingUDPTraffic(interval: TimeInterval) {
        sendDataTimer.schedule(deadline: .now(), repeating: interval)

        sendDataTimer.setEventHandler {
            let data = "dGhpcyBpcyBqdXN0IHNvbWUgZHVtbXkgZGF0YSB0aGlzIGlzIGp1c3Qgc29tZSBkdW".data(using: .utf8)

            print("Attempting to send data...")

            if self.connection.state != .ready {
                print("Not connected, won't send data")
            } else {
                self.connection.send(content: data, completion: .contentProcessed { error in
                    if let error = error {
                        print("Failed to send data: \(error)")
                    } else {
                        print("Data sent")
                    }
                })
            }
        }

        sendDataTimer.activate()
    }

    public func stopGeneratingUDPTraffic() {
        sendDataTimer.cancel()
    }
}
