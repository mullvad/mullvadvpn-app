//
//  ActorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadREST
import MullvadSettings
import MullvadTypes
import Network
@testable import PacketTunnelCore
@testable import RelaySelector
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import XCTest

final class ActorTests: XCTestCase {
    private var actor: PacketTunnelActor!
    private var stateSink: Combine.Cancellable?

    func testStart() async throws {
        let relaySelector = MockRelaySelector { constraints, failureCount in
            let publicKey = PrivateKey().publicKey.rawValue
            return RelaySelectorResult(
                endpoint: MullvadEndpoint(
                    ipv4Relay: IPv4Endpoint(ip: .loopback, port: 1300),
                    ipv4Gateway: .loopback,
                    ipv6Gateway: .loopback,
                    publicKey: publicKey
                ),
                relay: REST.ServerRelay(
                    hostname: "",
                    active: true,
                    owned: true,
                    location: "se",
                    provider: "",
                    weight: 0,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: publicKey,
                    includeInCountry: true
                ),
                location: Location(country: "", countryCode: "se", city: "", cityCode: "", latitude: 0, longitude: 0)
            )
        }

        let settingsReader = MockSettingsReader {
            return Settings(
                tunnelSettings: LatestTunnelSettings(
                    relayConstraints: RelayConstraints(),
                    dnsSettings: DNSSettings()
                ),
                deviceState: .loggedIn(
                    StoredAccountData(identifier: "", number: "123", expiry: .distantFuture),
                    StoredDeviceData(
                        creationDate: Date(), identifier: "", name: "", hijackDNS: false,
                        ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
                        ipv6Address: IPAddressRange(from: "::ff/64")!,
                        wgKeyData: StoredWgKeyData(
                            creationDate: Date(),
                            lastRotationAttemptDate: nil,
                            privateKey: PrivateKey(),
                            nextPrivateKey: nil
                        )
                    )
                )
            )
        }

        let tunnelMonitor = MockTunnelMonitor { command, dispatcher in
            switch command {
            case .start:
                // Dispatch reachability as it would normally happen in production.
                dispatcher.send(.networkReachabilityChanged(true))

                // Broadcast that connection was established after a short delay.
                dispatcher.send(.connectionEstablished, after: .milliseconds(100))

            case .stop:
                break
            }
        }

        actor = PacketTunnelActor(
            tunnelAdapter: MockTunnelAdapter(),
            tunnelMonitor: tunnelMonitor,
            relaySelector: relaySelector,
            settingsReader: settingsReader
        )

        /*
         As actor starts it should transition through the following states based on simulation:

         .initial → .connecting (reachability undetermined) → .connecting (reachability reachable) → .connected
         */
        let initialStateExpectation = expectation(description: "Expect initial state")
        let connectingNetUndeterminedStateExpectation =
            expectation(description: "Expect connecting state (network reachability: undetermined)")
        let connectingNetReachableExpectation =
            expectation(description: "Expect connecting state (network reachability: reachable)")
        let connectedStateExpectation = expectation(description: "Expect connected state")

        let allExpectations = [
            initialStateExpectation,
            connectingNetUndeterminedStateExpectation,
            connectingNetReachableExpectation,
            connectedStateExpectation,
        ]

        /*
         Monitor actor state. Initial value is always received upon creating a sink.
         */
        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .initial:
                    initialStateExpectation.fulfill()

                case let .connecting(connectingState) where connectingState.networkReachability == .undetermined:
                    connectingNetUndeterminedStateExpectation.fulfill()

                case let .connecting(connectingState) where connectingState.networkReachability == .reachable:
                    connectingNetReachableExpectation.fulfill()

                case .connected:
                    connectedStateExpectation.fulfill()

                default:
                    break
                }
            }

        try await actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 20, enforceOrder: true)
    }
}
