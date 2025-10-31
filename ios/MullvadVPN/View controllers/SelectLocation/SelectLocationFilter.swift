import SwiftUI

enum SelectLocationFilter: Hashable {
    case daita
    case obfuscation
    case owned
    case rented
    case provider(Int)

    var canBeRemoved: Bool {
        switch self {
        case .daita, .obfuscation:
            return false
        case .provider, .owned, .rented:
            return true
        }
    }

    var title: LocalizedStringKey {
        switch self {
        case .daita:
            return "Setting: \("DAITA")"
        case .obfuscation:
            return "Setting: \("Obfuscation")"
        case .owned:
            return "Owned"
        case .rented:
            return "Rented"
        case .provider(let count):
            return "Providers: \(count)"
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
}
