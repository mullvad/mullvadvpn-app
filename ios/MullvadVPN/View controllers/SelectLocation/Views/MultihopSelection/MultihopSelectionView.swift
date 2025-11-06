import SwiftUI

struct MultihopSelectionView: View {
    let hops: [Hop]
    @Binding var selectedMultihopContext: MultihopContext
    let deviceLocationName: String?
    let isExpanded: Bool

    @State private var animationId: UUID = .init()
    @Namespace private var animation
    @State private var pressedMultihopContext: MultihopContext?

    @State private var iconPositions: [AnyHashable: CGRect] = [:]
    private let spacing: CGFloat = 4
    private var outerHorizontalPadding: CGFloat {
        hops.count > 1 ? 4 : 0
    }

    @State private var viewHeight: CGFloat = 0
    @State private var topHeight: CGFloat = 0
    @State private var bottomHeight: CGFloat = 0
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if isExpanded {
                MultihopLabel(
                    label: "Internet",
                    image: Image.mullvadIconInternet,
                    onIconPositionChange: { position in
                        iconPositions["internet"] = position
                    }
                )
                .padding(.horizontal, outerHorizontalPadding + 8 + 2)
            }
            VStack(alignment: .leading, spacing: 3) {
                VStack(alignment: .leading, spacing: spacing) {
                    ForEach(
                        Array(hops.reversed().enumerated()),
                        id: \.element.multihopContext
                    ) {
                        index,
                        hop in
                        let isSelected = hop.multihopContext == selectedMultihopContext
                        ZStack(alignment: .topLeading) {
                            VStack(alignment: .leading, spacing: 2) {
                                PressedExposingButton {
                                    withAnimation {
                                        selectedMultihopContext = hop.multihopContext
                                    }
                                } label: {
                                    HopView(
                                        hop: hop,
                                        isSelected: selectedMultihopContext == hop.multihopContext,
                                        onFilterTapped: {
                                        },
                                        onIconPositionChange: { position in
                                            iconPositions[hop.multihopContext] = position
                                        }
                                    )
                                    .background {
                                        ZStack {
                                            if isSelected {
                                                RoundedRectangle(cornerRadius: 12)
                                                    .fill(Color.MullvadList.Item.child3)
                                                    .matchedGeometryEffect(id: animationId, in: animation)
                                            }
                                            if hop.noMatchFound != nil {
                                                RoundedRectangle(cornerRadius: 12)
                                                    .inset(by: 1)
                                                    .stroke(Color.mullvadDangerColor)
                                            }
                                        }
                                    }
                                    .contentShape(Rectangle())

                                } onPressedChange: {
                                    pressedMultihopContext = $0 ? hop.multihopContext : nil
                                }
                                .accessibilityIdentifier(hop.multihopContext.accessibilityIdentifier)
                                .accessibilityLabel(hop.multihopContext.description)
                                .disabled(hops.count == 1)
                                if let noMatchFound = hop.noMatchFound {
                                    Text(noMatchFound.description)
                                        .padding(.leading, 34)
                                        .foregroundStyle(Color.mullvadDangerColor)
                                        .font(.mullvadMini)
                                }
                            }
                        }
                    }
                }
                .padding(.horizontal, outerHorizontalPadding)
                .padding(.vertical, outerHorizontalPadding + 2)
                .background {
                    if hops.count > 1 {
                        Color.mullvadContainerBackground
                            .clipShape(RoundedRectangle(cornerRadius: 16))
                            .padding(.vertical, 2)
                    }
                }
                .zIndex(1)
                if isExpanded {
                    var label: LocalizedStringKey {
                        if let deviceLocationName {
                            "Your device (\(deviceLocationName))"
                        } else {
                            "Your device"
                        }
                    }
                    MultihopLabel(
                        label: label,
                        image: Image.mullvadSmartphone,
                        onIconPositionChange: { position in
                            withAnimation {
                                iconPositions["device"] = position
                            }
                        }
                    )
                    .transition(.move(edge: .top).combined(with: .opacity))
                    .padding(.horizontal, outerHorizontalPadding + 8 + 2)
                }
            }
            .geometryGroup()
        }
        .geometryGroup()
        .overlay {
            LineOverlayView(
                iconPositions: iconPositions,
                isExpanded: isExpanded
            )
        }
        .coordinateSpace(.multihopSelection)
    }
}

#Preview {
    @Previewable @State var selectedContext: MultihopContext = .allCases.first!
    @Previewable @State var isExpanded: Bool = true
    ScrollView {
        Button("Expanded") {
            withAnimation(.default.speed(0.2)) {
                isExpanded.toggle()
            }
        }
        VStack {
            Spacer()
            MultihopSelectionView(
                hops: [.init(multihopContext: .exit, selectedLocation: nil)],
                selectedMultihopContext: .constant(.exit),
                deviceLocationName: nil,
                isExpanded: isExpanded
            )
            MultihopSelectionView(
                hops: MultihopContext.allCases
                    .map {
                        Hop(
                            multihopContext: $0,
                            selectedLocation: .init(name: "\($0.description)", code: "se"))
                    },
                selectedMultihopContext: $selectedContext,
                deviceLocationName: "Sweden",
                isExpanded: isExpanded
            )
            .padding()
            MultihopSelectionView(
                hops: MultihopContext.allCases
                    .map {
                        Hop(
                            multihopContext: $0,
                            selectedLocation: nil
                        )
                    },
                selectedMultihopContext: $selectedContext,
                deviceLocationName: "Sweden",
                isExpanded: isExpanded
            )
            .padding()
            Spacer()
        }
    }
    .background(Color.mullvadBackground)
}

struct PressedExposingButton<Content: View>: View {
    let action: () -> Void
    let label: () -> Content
    let onPressedChange: ((Bool) -> Void)?
    struct MyButtonStyle: ButtonStyle {
        let action: () -> Void
        let label: () -> Content
        let onPressedChange: ((Bool) -> Void)?

        func makeBody(configuration: Configuration) -> some View {
            configuration.label
                .onChange(of: configuration.isPressed) {
                    onPressedChange?(configuration.isPressed)
                }
                .opacity(configuration.isPressed ? 0.6 : 1.0)
        }
    }

    var body: some View {
        Button(action: action, label: label)
            .buttonStyle(
                MyButtonStyle(
                    action: action,
                    label: label,
                    onPressedChange: onPressedChange
                )
            )
    }
}
