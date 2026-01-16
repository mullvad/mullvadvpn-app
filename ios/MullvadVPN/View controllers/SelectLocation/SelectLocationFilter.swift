import MullvadSettings
import SwiftUI

enum SelectLocationFilter: Hashable {
    case daita
    case obfuscation
    case ipv6
    case owned
    case rented
    case provider(Int)

    var isRemovable: Bool {
        switch self {
        case .daita, .obfuscation, .ipv6:
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
        case .ipv6:
            "Setting: \("IPv6")"
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
        case .ipv6:
            .ipv6FilterPill
        case .owned, .rented, .provider:
            .selectLocationFilterButton
        }
    }

    static func getActiveFilters(_ settings: LatestTunnelSettings) -> (
        [SelectLocationFilter],
        [SelectLocationFilter]
    ) {
        var activeEntryFilter: [SelectLocationFilter] = []
        var activeExitFilter: [SelectLocationFilter] = []

        let isMultihop = settings.tunnelMultihopState.isEnabled
        if let ownershipFilter = settings.relayConstraints.filter.value {
            switch ownershipFilter.ownership {
            case .any:
                break
            case .owned:
                activeEntryFilter.append(.owned)
                activeExitFilter.append(.owned)
            case .rented:
                activeEntryFilter.append(.rented)
                activeExitFilter.append(.rented)
            }
            if let provider = ownershipFilter.providers.value {
                activeEntryFilter.append(.provider(provider.count))
                activeExitFilter.append(.provider(provider.count))
            }
        }
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

        // Show IPv6 filter when IPv6 is selected AND obfuscation (shadowsocks/quic) is active
        // because regular entry IPv6 addresses don't work with these obfuscation methods
        if settings.ipVersion.isIPv6 && isObfuscation {
            if isMultihop {
                activeEntryFilter.append(.ipv6)
            } else {
                activeExitFilter.append(.ipv6)
            }
        }

        return (activeEntryFilter, activeExitFilter)
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
