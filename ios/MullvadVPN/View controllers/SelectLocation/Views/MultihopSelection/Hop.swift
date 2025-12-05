import SwiftUI

struct Hop {
    let multihopContext: MultihopContext
    let selectedLocation: LocationNode?
    var noMatchFound: NoMatchFoundReason? {
        if let selectedLocation {
            let userSelectedRelays = selectedLocation.userSelectedRelays
            if userSelectedRelays.customListSelection != nil,
                userSelectedRelays.locations.isEmpty
            {
                return .customListEmpty
            }
            return selectedLocation.isActive ? nil : .selectionInactive
        }
        return .noSelection
    }

    var icon: Image {
        if noMatchFound != nil {
            return Image.mullvadIconError
        }
        return switch multihopContext {
        case .entry:
            Image.mullvadServer
        case .exit:
            Image.mullvadLocation
        }
    }

    enum NoMatchFoundReason {
        // Previous selection is no longer valid with filters, settings or simply disappeared from the relay list
        case noSelection
        // A location is inactive when all containing relays are inactive (offline)
        case selectionInactive
        // A selected custom list has no locations with current filters settings or is simply empty.
        case customListEmpty

        // Could me more detailed in the future
        var description: LocalizedStringKey {
            "No servers match your settings, try changing server or other settings."
        }
    }
}
