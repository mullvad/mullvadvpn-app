import MullvadTypes
import SwiftUI

struct LocationItemView: View {
    let node: LocationNode
    let multihopContext: MultihopContext
    let level: Int

    var isDisabled: Bool {
        !node.isActive || node.isExcluded
    }

    var subtitle: String? {
        if node.isConnected {
            if let node = node.asAutomaticLocationNode {
                node.locationInfo?.joined(separator: ", ")
            } else if !node.isSelected {
                String(localized: "Connected server")
            } else {
                nil
            }
        } else {
            nil
        }
    }

    @ViewBuilder var statusIndicator: some View {
        let itemFactory = ListItemFactory()

        if !node.isActive {
            itemFactory.statusIndicator(for: .dot(.offline))
        } else if node.isSelected {
            itemFactory.statusIndicator(for: .tick)
        } else {
            EmptyView()
        }
    }

    var title: String {
        if node.isExcluded {
            switch multihopContext {
            case .entry:
                return """
                    \(node.name) (\(String(localized:
                    String
                    .LocalizationValue(MultihopContext.exit.description))))
                    """
            case .exit:
                return """
                    \(node.name) (\(String(localized:
                    String
                    .LocalizationValue(MultihopContext.entry.description))))
                    """
            }
        }
        return "\(node.name)"
    }

    var body: some View {
        ListItem(
            title: title,
            subtitle: subtitle,
            level: level,
            selected: node.isSelected,
            statusIndicator: { statusIndicator }
        )
        .disabled(isDisabled)
    }
}

#Preview {
    LocationItemView(
        node: LocationNode(
            name: "A great location",
            code: "a-great-location",
            isSelected: true
        ),
        multihopContext: .exit,
        level: 0
    )
}
