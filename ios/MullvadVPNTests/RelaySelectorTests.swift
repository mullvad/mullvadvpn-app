//
//  RelaySelectorTests.swift
//  RelaySelectorTests
//
//  Created by pronebird on 07/11/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import XCTest
import Network

class RelaySelectorTests: XCTestCase {

    func testCountryConstraint() {
        let relaySelector = RelaySelector(relayList: sampleRelayList)
        let constraints = RelayConstraints(location: .only(.country("es")))

        let result = relaySelector.evaluate(with: constraints)

        XCTAssertEqual(result?.relay.hostname, "es1-wireguard")
    }

    func testCityConstraint() {
        let relaySelector = RelaySelector(relayList: sampleRelayList)
        let constraints = RelayConstraints(location: .only(.city("se", "got")))

        let result = relaySelector.evaluate(with: constraints)

        XCTAssertEqual(result?.relay.hostname, "se10-wireguard")
    }

    func testHostnameConstraint() {
        let relaySelector = RelaySelector(relayList: sampleRelayList)
        let constraints = RelayConstraints(location: .only(.hostname("se", "sto", "se6-wireguard")))

        let result = relaySelector.evaluate(with: constraints)

        XCTAssertEqual(result?.relay.hostname, "se6-wireguard")
    }

}

private let sampleRelayList = RelayList(countries: [
    .init(name: "Spain", code: "es", cities: [
        .init(name: "Madrid",
              code: "mad",
              latitude: 40.408566,
              longitude: -3.69222,
              relays: [
                .init(
                    hostname: "es1-wireguard",
                    ipv4AddrIn: .loopback,
                    includeInCountry: true,
                    active: true,
                    weight: 0,
                    tunnels: .init(wireguard: [
                        .init(
                            ipv4Gateway: .loopback,
                            ipv6Gateway: .loopback,
                            publicKey: .init(),
                            portRanges: [(7000...7100)]
                        )
                    ]))
        ])
    ]),
    .init(name: "Sweden", code: "se", cities: [
        .init(name: "Gothenburg",
              code: "got",
              latitude: 57.70887,
              longitude: 11.97456,
              relays: [
                .init(
                    hostname: "se10-wireguard",
                    ipv4AddrIn: .loopback,
                    includeInCountry: true,
                    active: true,
                    weight: 0,
                    tunnels: .init(wireguard: [
                        .init(
                            ipv4Gateway: .loopback,
                            ipv6Gateway: .loopback,
                            publicKey: .init(),
                            portRanges: [(7000...7100)]
                        )
                    ]))
        ]),
        .init(name: "Stockholm",
              code: "sto",
              latitude: 59.3289,
              longitude: 18.0649,
              relays: [
                .init(
                    hostname: "se2-wireguard",
                    ipv4AddrIn: .loopback,
                    includeInCountry: true,
                    active: true,
                    weight: 0,
                    tunnels: .init(wireguard: [
                        .init(
                            ipv4Gateway: .loopback,
                            ipv6Gateway: .loopback,
                            publicKey: .init(),
                            portRanges: [(8000...8100)]
                        )
                    ])),
                .init(
                    hostname: "se6-wireguard",
                    ipv4AddrIn: IPv4Address.loopback,
                    includeInCountry: true,
                    active: true,
                    weight: 0,
                    tunnels: .init(wireguard: [
                        .init(
                            ipv4Gateway: .loopback,
                            ipv6Gateway: .loopback,
                            publicKey: .init(),
                            portRanges: [(8000...9000)]
                        )
                    ]))
        ])
    ])
])
