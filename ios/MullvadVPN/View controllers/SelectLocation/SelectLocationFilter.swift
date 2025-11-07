import MullvadSettings
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
        default: nil
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
