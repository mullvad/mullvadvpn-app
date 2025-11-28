import MullvadSettings
import MullvadTypes
import SwiftUI

enum SelectLocationFilter: Hashable {
    case daita
    case obfuscation
    case owned
    case rented
    case provider(Int)

    var isRemovable: Bool {
        switch self {
        case .daita, .obfuscation:
            false
        case .provider, .owned, .rented:
            true
        }
    }

    var title: LocalizedStringKey {
        switch self {
        case .daita:
            "Setting: \("DAITA")"
        case .obfuscation:
            "Setting: \("Obfuscation")"
        case .owned:
            "Owned"
        case .rented:
            "Rented"
        case .provider(let count):
            "Providers: \(count)"
        }
    }

    var accessibilityIdentifier: AccessibilityIdentifier? {
        switch self {
        case .daita:
            .daitaFilterPill
        case .obfuscation:
            .obfuscationFilterPill
        case .owned, .rented, .provider:
            .selectLocationFilterButton
        }
    }

    static func getActiveFilters(_ settings: LatestTunnelSettings) -> (
        [SelectLocationFilter],
        [SelectLocationFilter]
    ) {
        var activeEntryFilter = [SelectLocationFilter]()
        add(relayFilter: settings.relayConstraints.entryFilter, to: &activeEntryFilter)

        var activeExitFilter = [SelectLocationFilter]()
        add(relayFilter: settings.relayConstraints.exitFilter, to: &activeExitFilter)

        let isMultihop = settings.tunnelMultihopState.isEnabled

        if settings.daita.isDirectOnly {
            if isMultihop {
                activeEntryFilter.append(.daita)
            } else {
                activeExitFilter.append(.daita)
            }
        }

        let isObfuscation = settings.wireGuardObfuscation.state.affectsRelaySelection
        if isObfuscation {
            if isMultihop {
                activeEntryFilter.append(.obfuscation)
            } else {
                activeExitFilter.append(.obfuscation)
            }
        }

        return (activeEntryFilter, activeExitFilter)
    }

    private static func add(
        relayFilter: RelayConstraint<RelayFilter>,
        to locationFilter: inout [SelectLocationFilter]
    ) {
        if let relayFilter = relayFilter.value {
            switch relayFilter.ownership {
            case .any:
                break
            case .owned:
                locationFilter.append(.owned)
            case .rented:
                locationFilter.append(.rented)
            }
            if let provider = relayFilter.providers.value {
                locationFilter.append(.provider(provider.count))
            }
        }
    }
}

private extension WireGuardObfuscationState {
    /// This flag affects whether the "Setting: Obfuscation" pill is shown when selecting a location
    var affectsRelaySelection: Bool {
        switch self {
        case .shadowsocks, .quic:
            true
        default: false
        }
    }
}
