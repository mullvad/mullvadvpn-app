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
}
