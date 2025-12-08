import SwiftUI

private enum ViewPositionIdentifiers: Hashable {
    case internet
    case yourDevice
}

struct MultihopSelectionView: View {
    let hops: [Hop]
    @Binding var selectedMultihopContext: MultihopContext
    let isExpanded: Bool
    var deviceLocationName: String?

    @State private var animationIdSelection: UUID = .init()
    @State private var animationIdBackground: UUID = .init()
    @Namespace private var animation
    @State private var pressedMultihopContext: MultihopContext?

    @State private var iconPositions: [AnyHashable: CGRect] = [:]

    var filteredIconPositions: [AnyHashable: CGRect] {
        let allowedKeys: [AnyHashable] =
            hops.map { $0.multihopContext }
            + [
                ViewPositionIdentifiers.internet as AnyHashable,
                ViewPositionIdentifiers.yourDevice as AnyHashable,
            ]
        return
            iconPositions
            .filter {
                allowedKeys.contains($0.key)
            }
    }

    private let spacing: CGFloat = 4
    private var outerPadding: CGFloat {
        hops.count > 1 ? 4 : 0
    }

    @State private var viewHeight: CGFloat = 0
    @State private var topHeight: CGFloat = 0
    @State private var bottomHeight: CGFloat = 0
    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            if isExpanded {
                MultihopLabel(
                    label: "Internet",
                    image: Image.mullvadIconInternet,
                    onIconPositionChange: { position in
                        iconPositions[ViewPositionIdentifiers.internet] = position
                    }
                )
                .padding(.horizontal, outerPadding + 8 + 2)
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
                                                    .matchedGeometryEffect(id: animationIdSelection, in: animation)
                                                    .background {
                                                        if hops.count == 1 {
                                                            Color.clear
                                                                .matchedGeometryEffect(
                                                                    id: animationIdBackground,
                                                                    in: animation
                                                                )
                                                        }
                                                    }
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
                        .zIndex(1 / Double(index + 1))
                        .transition(.move(edge: .top).combined(with: .opacity))
                    }
                }
                .padding(.horizontal, outerPadding)
                .padding(.vertical, outerPadding)
                .background {
                    if hops.count > 1 {
                        Color.mullvadContainerBackground
                            .clipShape(RoundedRectangle(cornerRadius: 16))
                            .matchedGeometryEffect(
                                id: animationIdBackground,
                                in: animation
                            )
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
                            iconPositions[ViewPositionIdentifiers.yourDevice] = position
                        }
                    )
                    .transition(.move(edge: .top).combined(with: .opacity))
                    .padding(.horizontal, outerPadding + 8 + 2)
                }
            }
            .geometryGroup()
        }
        .coordinateSpace(.multihopSelection)
        .overlay(alignment: .topLeading) {
            LineOverlayView(
                iconPositions: filteredIconPositions,
                isExpanded: isExpanded
            )
            .animation(nil, value: hops.count)
        }
        .geometryGroup()
        .animation(.default, value: hops.count)
        .animation(.default, value: isExpanded)
    }
}

#Preview {
    @Previewable @State var selectedContext: MultihopContext = .exit
    @Previewable @State var isExpanded: Bool = true
    @Previewable @State var contexts: [MultihopContext] = MultihopContext.allCases
    ScrollView {
        Button("Expanded") {
            isExpanded.toggle()
        }
        Button("Toggle multihop") {
            if contexts.count == 1 {
                contexts = MultihopContext.allCases
            } else {
                contexts = [.exit]
            }
        }
        VStack(spacing: 8) {
            Spacer()
            MultihopSelectionView(
                hops: [.init(multihopContext: .exit, selectedLocation: nil)],
                selectedMultihopContext: .constant(.exit),
                isExpanded: isExpanded
            )
            .padding()
            MultihopSelectionView(
                hops:
                    contexts
                    .map {
                        Hop(
                            multihopContext: $0,
                            selectedLocation: .init(name: "\($0.description)", code: "se"))
                    },
                selectedMultihopContext: $selectedContext,
                isExpanded: isExpanded,
                deviceLocationName: "Sweden"
            )
            .padding()
            MultihopSelectionView(
                hops:
                    contexts
                    .map {
                        Hop(
                            multihopContext: $0,
                            selectedLocation: nil
                        )
                    },
                selectedMultihopContext: $selectedContext,
                isExpanded: isExpanded,
                deviceLocationName: "Sweden"
            )
            .padding()
            Spacer()
        }
    }
    .background(Color.mullvadDarkBackground)
}
