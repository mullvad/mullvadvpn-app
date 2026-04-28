//
//  MultihopDataGatherer.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Testing

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

// Not meant to be run as a test, only for gathering data about multihop relay selection.

class MultihopDataGatherer {
    private var output = ""
    private let outputURL: URL
    private let relays: REST.ServerRelaysResponse

    init() throws {
        let projectDir = URL(fileURLWithPath: #filePath.components(separatedBy: "MullvadVPNTests")[0])
        outputURL = projectDir.appendingPathComponent("multihop-data.csv")

        let data = try Data(contentsOf: URL(string: "https://api.mullvad.net/app/v1/relays")!)
        relays = try REST.Coding.makeJSONDecoder().decode(REST.ServerRelaysResponse.self, from: data)
    }

    @Test func gatherMultihopData() throws {
        output = "Multihop,Obfuscation,Exit location,Exit server,Entry location,Entry server,Distance (km)\n"

        appendMultihopData(obfuscationState: .lwo, multihopState: .always)
        appendMultihopData(obfuscationState: .quic, multihopState: .always)
        appendMultihopData(obfuscationState: .lwo, multihopState: .whenNeeded)
        appendMultihopData(obfuscationState: .quic, multihopState: .whenNeeded)

        print("\n----------\n")
        print(output)
        print("----------\n")

        try output.write(to: outputURL, atomically: true, encoding: .utf8)
    }

    private func appendMultihopData(
        obfuscationState: WireGuardObfuscationState,
        multihopState: MultihopState
    ) {
        var settings = LatestTunnelSettings(tunnelMultihopState: multihopState, daita: .init(daitaState: .on))
        settings.wireGuardObfuscation.state = obfuscationState

        let relayLocations = RelayWithLocation.locateRelays(
            relays: relays.wireguard.relays,
            locations: relays.locations
        )

        for relayWithLocation in relayLocations {
            guard relayWithLocation.relay.active else { continue }

            let splitLocation = relayWithLocation.relay.hostname.split(separator: "-")
            guard
                let location = RelayLocation(
                    dashSeparatedString: "\(splitLocation[0])-\(splitLocation[1])-\(relayWithLocation.relay.hostname)"
                )
            else { continue }

            let selectedRelay = UserSelectedRelays(locations: [location])
            settings.relayConstraints.exitLocations = .only(selectedRelay)

            let picker: RelayPicking
            if multihopState == .always {
                settings.relayConstraints.entryLocations = .any

                picker = MultihopPicker(
                    relays: relays,
                    tunnelSettings: settings,
                    connectionAttemptCount: 0
                )
            } else {
                picker = SinglehopPicker(
                    relays: relays,
                    tunnelSettings: settings,
                    connectionAttemptCount: 0
                )
            }

            do {
                let routedRelays = try picker.pick()
                if let routedEntry = routedRelays.entry {
                    let distance = Haversine.distance(
                        relayWithLocation.serverLocation.latitude,
                        relayWithLocation.serverLocation.longitude,
                        routedEntry.location.latitude,
                        routedEntry.location.longitude
                    )

                    output +=
                        "\(multihopState),"
                        + "\(obfuscationState),"
                        + "\(csvEscape("\(relayWithLocation.serverLocation.country), \(relayWithLocation.serverLocation.city)")),"
                        + "\(relayWithLocation.relay.hostname),"
                        + "\(csvEscape("\(routedEntry.location.country), \(routedEntry.location.city)")),"
                        + "\(routedEntry.hostname),"
                        + "\(String(format: "%.0f", distance))\n"
                }
            } catch {}
        }
    }

    private func csvEscape(_ value: String) -> String {
        let stripped = value.replacingOccurrences(of: "\"", with: "")
        return stripped.contains(",") ? "\"\(stripped)\"" : stripped
    }
}
