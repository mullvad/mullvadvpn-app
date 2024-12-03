//
//  RelaySelectorPicker.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-06-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Network

protocol RelayPicking {
    var obfuscation: ObfuscatorPortSelection { get }
    var constraints: RelayConstraints { get }
    var connectionAttemptCount: UInt { get }
    var daitaSettings: DAITASettings { get }
    func pick() throws -> SelectedRelays
}

extension RelayPicking {
    func findBestMatch(
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        useObfuscatedPortIfAvailable: Bool
    ) throws -> SelectedRelay {
        var match = try RelaySelector.WireGuard.pickCandidate(
            from: candidates,
            wireguard: obfuscation.wireguard,
            portConstraint: useObfuscatedPortIfAvailable ? obfuscation.port : constraints.port,
            numberOfFailedAttempts: connectionAttemptCount,
            closeTo: location
        )

        if useObfuscatedPortIfAvailable && obfuscation.method == .shadowsocks {
            match = applyShadowsocksIpAddress(in: match)
        }

        return SelectedRelay(
            endpoint: match.endpoint,
            hostname: match.relay.hostname,
            location: match.location
        )
    }

    private func applyShadowsocksIpAddress(in match: RelaySelectorMatch) -> RelaySelectorMatch {
        let port = match.endpoint.ipv4Relay.port
        let portRanges = RelaySelector.parseRawPortRanges(obfuscation.wireguard.shadowsocksPortRanges)
        let portIsWithinRange = portRanges.contains(where: { $0.contains(port) })

        var endpoint = match.endpoint

        // If the currently selected obfuscation port is not within the allowed range (as specified
        // in the relay list), we should use one of the extra Shadowsocks IP addresses instead of
        // the default one.
        if !portIsWithinRange {
            var ipv4Address = match.endpoint.ipv4Relay.ip
            if let shadowsocksAddress = match.relay.shadowsocksExtraAddrIn?.randomElement() {
                ipv4Address = IPv4Address(shadowsocksAddress) ?? ipv4Address
            }

            endpoint = match.endpoint.override(ipv4Relay: IPv4Endpoint(
                ip: ipv4Address,
                port: port
            ))
        }

        return RelaySelectorMatch(endpoint: endpoint, relay: match.relay, location: match.location)
    }
}

struct SinglehopPicker: RelayPicking {
    let obfuscation: ObfuscatorPortSelection
    let constraints: RelayConstraints
    let connectionAttemptCount: UInt
    let daitaSettings: DAITASettings

    func pick() throws -> SelectedRelays {
        do {
            let exitCandidates = try RelaySelector.WireGuard.findCandidates(
                by: constraints.exitLocations,
                in: obfuscation.exitRelays,
                filterConstraint: constraints.filter,
                daitaEnabled: daitaSettings.daitaState.isEnabled
            )

            let match = try findBestMatch(from: exitCandidates, useObfuscatedPortIfAvailable: true)
            return SelectedRelays(entry: nil, exit: match, retryAttempt: connectionAttemptCount)
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
            // If DAITA is on and Direct only is off, and no supported relays are found, we should try to find the nearest
            // available relay that supports DAITA and use it as entry in a multihop selection.
            if daitaSettings.isAutomaticRouting {
                return try MultihopPicker(
                    obfuscation: obfuscation,
                    constraints: constraints,
                    connectionAttemptCount: connectionAttemptCount,
                    daitaSettings: daitaSettings
                ).pick()
            } else {
                throw error
            }
        }
    }
}

struct MultihopPicker: RelayPicking {
    let obfuscation: ObfuscatorPortSelection
    let constraints: RelayConstraints
    let connectionAttemptCount: UInt
    let daitaSettings: DAITASettings

    func pick() throws -> SelectedRelays {
        let exitCandidates = try RelaySelector.WireGuard.findCandidates(
            by: constraints.exitLocations,
            in: obfuscation.exitRelays,
            filterConstraint: constraints.filter,
            daitaEnabled: false
        )

        /*
         Relay selection is prioritised in the following order:
         1. Both entry and exit constraints match only a single relay. Both relays are selected.
         2. Entry constraint matches only a single relay and the other multiple relays. The single relay
            is selected and excluded from the list of multiple relays.
         3. Exit constraint matches multiple relays and the other a single relay. The single relay
            is selected and excluded from the list of multiple relays.
         4. Both entry and exit constraints match multiple relays. Exit relay is picked first and then
            excluded from the list of entry relays.
         */
        let decisionFlow = OneToOne(
            next: OneToMany(
                next: ManyToOne(
                    next: ManyToMany(
                        next: nil,
                        relayPicker: self
                    ),
                    relayPicker: self
                ),
                relayPicker: self
            ),
            relayPicker: self
        )

        do {
            let entryCandidates = try RelaySelector.WireGuard.findCandidates(
                by: daitaSettings.isAutomaticRouting ? .any : constraints.entryLocations,
                in: obfuscation.entryRelays,
                filterConstraint: constraints.filter,
                daitaEnabled: daitaSettings.daitaState.isEnabled
            )

            return try decisionFlow.pick(
                entryCandidates: entryCandidates,
                exitCandidates: exitCandidates,
                daitaAutomaticRouting: daitaSettings.isAutomaticRouting
            )
        }
    }

    func exclude(
        relay: SelectedRelay,
        from candidates: [RelayWithLocation<REST.ServerRelay>],
        closeTo location: Location? = nil,
        useObfuscatedPortIfAvailable: Bool
    ) throws -> SelectedRelay {
        let filteredCandidates = candidates.filter { relayWithLocation in
            relayWithLocation.relay.hostname != relay.hostname
        }

        return try findBestMatch(
            from: filteredCandidates,
            closeTo: location,
            useObfuscatedPortIfAvailable: useObfuscatedPortIfAvailable
        )
    }
}
