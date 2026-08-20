import SwiftUI

struct MullvadListSectionHeader: View {
    struct Accessory: Identifiable {
        enum Face: Hashable {
            case text(String)
            case icon(ImageResource)
        }
        typealias ID = Face
        let face: Face
        let accessibilityId: AccessibilityIdentifier?
        let accessibilityLabel: LocalizedStringKey?
        let accessibilityHint: LocalizedStringKey?
        let action: () -> Void
        var id: ID { face }
    }
    let title: LocalizedStringKey
    let subtitle: LocalizedStringKey?
    let accessories: [Accessory]

    init(title: LocalizedStringKey, subtitle: LocalizedStringKey? = nil, accessories: [Accessory] = []) {
        self.title = title
        self.subtitle = subtitle
        self.accessories = accessories
    }

    var body: some View {
        HStack {
            Text(title)
                .font(.mullvadTinySemiBold)
                .foregroundStyle(Color.mullvadTextPrimary)
                .layoutPriority(1)
            Rectangle()
                .frame(height: 1)
                .foregroundStyle(Color.mullvadTextPrimary.opacity(0.2))
            if let subtitle {
                Text(subtitle)
                    .font(.mullvadTiny)
                    .foregroundStyle(Color.mullvadTextSecondary)
                    .layoutPriority(1)
            }
            ForEach(accessories) { accessory in
                accessoryView(accessory)
            }
        }
        .frame(minHeight: 44, alignment: .center)
        .accessibilityAddTraits(.isHeader)
    }

    func accessoryView(_ accessory: Accessory) -> some View {
        Button(
            action: accessory.action,
            label: { accessoryFaceView(accessory.face) }
        )
        .ifLet(accessory.accessibilityLabel) { $0.accessibilityLabel($1) }
        .ifLet(accessory.accessibilityHint) { $0.accessibilityHint($1) }
        .ifLet(accessory.accessibilityId) { $0.accessibilityIdentifier($1.asString) }
    }

    func accessoryFaceView(_ face: Accessory.Face) -> some View {
        switch face {
        case .icon(let imageResource):
            Image(imageResource)
                .renderingMode(.template)
                .resizable()
                .scaledToFit()
                .foregroundStyle(Color.mullvadTextPrimary)
                .frame(height: 24)
                .typeErase()
        case .text(let text):
            Text(text)
                .font(.mullvadTinySemiBold)
                .underline()
                .foregroundStyle(Color.mullvadTextPrimary)
                .typeErase()
        }
    }
}

// MARK: convenience initialisers for accessories
extension MullvadListSectionHeader.Accessory {
    init(
        _ text: String,
        accessibilityId: AccessibilityIdentifier? = nil,
        accessibilityLabel: LocalizedStringKey? = nil,
        accessibilityHint: LocalizedStringKey? = nil,
        action: @escaping () -> Void
    ) {
        self.init(
            face: Face.text(text),
            accessibilityId: accessibilityId,
            accessibilityLabel: accessibilityLabel,
            accessibilityHint: accessibilityHint,
            action: action
        )
    }

    init(
        _ icon: ImageResource,
        accessibilityId: AccessibilityIdentifier? = nil,
        accessibilityLabel: LocalizedStringKey? = nil,
        accessibilityHint: LocalizedStringKey? = nil,
        action: @escaping () -> Void
    ) {
        self.init(
            face: Face.icon(icon),
            accessibilityId: accessibilityId,
            accessibilityLabel: accessibilityLabel,
            accessibilityHint: accessibilityHint,
            action: action
        )
    }
}

#Preview {
    VStack {
        MullvadListSectionHeader(title: "Custom lists").background(Color.mullvadBackground)
        MullvadListSectionHeader(
            title: "Custom lists", subtitle: "Showing 32 of 194",
            accessories: [
                .init(
                    .iconReload,
                    action: {
                        print("Reload")
                    }),
                .init(
                    .iconAdd,
                    action: {
                        print("Add")
                    }),
            ]
        ).background(Color.mullvadBackground)
        MullvadListSectionHeader(
            title: "Custom lists",
            accessories: [
                .init(
                    "Text button",
                    action: {
                        print("Text button")
                    })
            ]
        ).background(Color.mullvadBackground)
    }
}
