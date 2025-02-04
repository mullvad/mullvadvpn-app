//
//  EphemeralPeerExchangeActorTests.swift
//  MullvadRustRuntimeTests
//
//  Created by Marco Nikic on 2024-06-12.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadRustRuntime
@testable import MullvadTypes
import NetworkExtension
@testable import PacketTunnelCore
@testable import WireGuardKitTypes
import XCTest

class EphemeralPeerExchangeActorTests: XCTestCase {
    var tcpConnection: NWTCPConnectionStub!
    var tunnelProvider: TunnelProviderStub!

    override func setUpWithError() throws {
        tcpConnection = NWTCPConnectionStub()
        tunnelProvider = TunnelProviderStub(tcpConnection: tcpConnection)
    }

    func testKeyExchangeFailsWhenNegotiationCannotStart() {
        let negotiationFailure = expectation(description: "Negotiation failed")

        let keyExchangeActor = EphemeralPeerExchangeActor(
            packetTunnel: tunnelProvider,
            onFailure: {
                negotiationFailure.fulfill()
            },
            negotiationProvider: FailedNegotiatorStub.self,
            iteratorProvider: { AnyIterator { .milliseconds(10) } }
        )

        let privateKey = PrivateKey()
        keyExchangeActor.startNegotiation(with: privateKey, enablePostQuantum: true, enableDaita: false)
        tcpConnection.becomeViable()

        wait(for: [negotiationFailure])
    }

    func testKeyExchangeFailsWhenTCPConnectionIsNotReadyInTime() {
        let negotiationFailure = expectation(description: "Negotiation failed")

        // Setup the actor to wait only 10 milliseconds before declaring the TCP connection is not ready in time.
        let keyExchangeActor = EphemeralPeerExchangeActor(
            packetTunnel: tunnelProvider,
            onFailure: {
                negotiationFailure.fulfill()
            },
            negotiationProvider: FailedNegotiatorStub.self,
            iteratorProvider: { AnyIterator { .milliseconds(10) } }
        )

        let privateKey = PrivateKey()
        keyExchangeActor.startNegotiation(with: privateKey, enablePostQuantum: true, enableDaita: false)

        wait(for: [negotiationFailure])
    }

    func testResetEndsTheCurrentNegotiation() throws {
        let unexpectedNegotiationFailure = expectation(description: "Unexpected negotiation failure")
        unexpectedNegotiationFailure.isInverted = true

        let keyExchangeActor = EphemeralPeerExchangeActor(
            packetTunnel: tunnelProvider,
            onFailure: {
                unexpectedNegotiationFailure.fulfill()
            },
            negotiationProvider: SuccessfulNegotiatorStub.self,
            iteratorProvider: { AnyIterator { .seconds(1) } }
        )

        let privateKey = PrivateKey()
        keyExchangeActor.startNegotiation(with: privateKey, enablePostQuantum: true, enableDaita: false)

        let negotiationProvider = try XCTUnwrap(
            keyExchangeActor.negotiation?
                .negotiator as? SuccessfulNegotiatorStub
        )

        let negotationCancelledExpectation = expectation(description: "Negotiation cancelled")
        negotiationProvider.onCancelKeyNegotiation = {
            negotationCancelledExpectation.fulfill()
        }

        keyExchangeActor.reset()

        wait(for: [negotationCancelledExpectation, unexpectedNegotiationFailure], timeout: .UnitTest.invertedTimeout)
    }
}
