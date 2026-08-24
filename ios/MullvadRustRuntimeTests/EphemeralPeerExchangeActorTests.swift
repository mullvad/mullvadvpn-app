// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import NetworkExtension
import XCTest

@testable import MullvadMockData
@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import PacketTunnelCore

class EphemeralPeerExchangeActorTests: XCTestCase {
    var tunnelProvider: TunnelProviderStub!

    override func setUpWithError() throws {
        RustLogging.initialize(logger: Logger(label: "Rust"))
        tunnelProvider = TunnelProviderStub()
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

        let privateKey = WireGuard.PrivateKey()
        keyExchangeActor.startNegotiation(with: privateKey, enablePostQuantum: true, enableDaita: false)

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

        let privateKey = WireGuard.PrivateKey()
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

        let privateKey = WireGuard.PrivateKey()
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
