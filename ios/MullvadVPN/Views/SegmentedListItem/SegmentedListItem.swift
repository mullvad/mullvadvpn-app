import SwiftUI

struct SegmentedListItem<Leading: View, Trailing: View, Segment: View, GroupedContent: View>: View {
    @State private var segmentHeight: CGFloat = UIMetrics.LocationList.cellMinHeight

    var level: Int = 0
    var isLastInList: Bool = true
    var isDisabled: Bool = false
    var accessibilityIdentifier: AccessibilityIdentifier?
    var accessibilityLabel: String = ""
    @ViewBuilder var leading: () -> Leading?
    @ViewBuilder var trailing: () -> Trailing?
    @ViewBuilder var segment: () -> Segment?
    @ViewBuilder var groupedContent: () -> GroupedContent?
    var footer: String? = nil
    var onSelect: (() -> Void)? = nil

    private var topRadius: CGFloat {
        level == 0 ? UIMetrics.LocationList.cellCornerRadius : 0
    }
    private var bottomRadius: CGFloat {
        isLastInList && (groupedContent() == nil || groupedContent() is EmptyView)
            ? UIMetrics.LocationList.cellCornerRadius
            : 0
    }

    var body: some View {
        HStack(spacing: 2) {
            Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    onSelect?()
                }
            } label: {
                let trailing = trailing()

                HStack(spacing: 8) {
                    leading()
                    Spacer()
                    trailing
                }
                .padding(.leading, 16)
                .padding(.trailing, trailing == nil ? 16 : 0)
                .background(Color.colorForLevel(level))
                .disabled(isDisabled)
                .sizeOfView {
                    segmentHeight = $0.height
                }
            }

            segment()?
                .frame(width: UIMetrics.LocationList.cellMinHeight, height: segmentHeight)
                .contentShape(Rectangle())
                .background(Color.colorForLevel(level))
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(accessibilityLabel)
        .accessibilityIdentifier(accessibilityIdentifier)
        .accessibilityAction(named: Text("Select \(accessibilityLabel)")) {
            withAnimation(.easeInOut(duration: 0.15)) {
                onSelect?()
            }
        }
        .clipShape(
            UnevenRoundedRectangle(
                cornerRadii: .init(
                    topLeading: topRadius,
                    bottomLeading: bottomRadius,
                    bottomTrailing: bottomRadius,
                    topTrailing: topRadius
                )
            )
        )
        .padding(.top, level == 0 ? 4 : 1)

        groupedContent()

        if let footer {
            HStack {
                Text(footer)
                    .font(.mullvadTiny)
                    .foregroundStyle(Color.MullvadText.onBackground)
                    .padding(.horizontal, 16)
                    .padding(.top, 2)
                Spacer()
            }
        }
    }
}

#Preview {
    @Previewable @State var inputText: String = ""
    @Previewable @State var toggleState: Bool = false

    let itemFactory = SegmentedListItemFactory()

    VStack(spacing: 0) {
        SegmentedListItem(
            leading: {
                itemFactory.leading(for: .setting(title: "Setting - custom"))
            },
            trailing: {
                itemFactory.trailing(
                    for: .custom(items: [
                        .breadcrumb(.warning(.root)),
                        .string("Custom text"),
                        .button(icon: .info, onSelect: { print("onSelect") }),
                        .button(icon: .close, onSelect: { print("onClose") }),
                    ])
                )
            },
            footer: "Short description instead of an info icon"
        )

        Spacer()
    }
    .background(Color.mullvadBackground)
}

// MARK: - Color helpers

private extension Color {
    static func colorForLevel(_ level: Int) -> Color {
        switch level {
        case 1: Color.MullvadList.Item.child1
        case 2: Color.MullvadList.Item.child2
        case 3: Color.MullvadList.Item.child3
        case 4: Color.MullvadList.Item.child4
        default: Color.MullvadList.Item.parent
        }
    }
}
