//
//  ChipContainerView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipContainerView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel
    let tunnelState: TunnelState
    @Binding var isExpanded: Bool

    @State private var chipContainerHeight: CGFloat = .zero
    private let verticalPadding: CGFloat = 8

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width

            let (chipsToAdd, showMoreButton) = if isExpanded {
                (viewModel.chips, false)
            } else {
                viewModel.chipsToAdd(forContainerWidth: containerWidth)
            }

            HStack {
                ZStack(alignment: .topLeading) {
                    createChipViews(chips: chipsToAdd, containerWidth: containerWidth)
                }

                Button(LocalizedStringKey("\(viewModel.chips.count - chipsToAdd.count) more...")) {
                    withAnimation {
                        isExpanded.toggle()
                    }
                }
                .font(.subheadline)
                .lineLimit(1)
                .foregroundStyle(UIColor.primaryTextColor.color)
                .showIf(showMoreButton)
                .transition(.move(edge: .bottom).combined(with: .opacity))

                Spacer()
            }
            .sizeOfView { size in
                withAnimation {
                    chipContainerHeight = size.height
                }
            }
        }
        .frame(height: chipContainerHeight)
    }

    private func createChipViews(chips: [ChipModel], containerWidth: CGFloat) -> some View {
        nonisolated(unsafe) var width = CGFloat.zero
        nonisolated(unsafe) var height = CGFloat.zero

        return ForEach(chips) { data in
            ChipView(item: data)
                .padding(
                    EdgeInsets(
                        top: verticalPadding,
                        leading: 0,
                        bottom: verticalPadding,
                        trailing: UIMetrics.FeatureIndicators.chipViewTrailingMargin
                    )
                )
                .alignmentGuide(.leading) { dimension in
                    if abs(width - dimension.width) > containerWidth {
                        width = 0
                        height -= dimension.height
                    }
                    let result = width
                    if data.id == chips.last?.id {
                        width = 0
                    } else {
                        width -= dimension.width
                    }
                    return result
                }
                .alignmentGuide(.top) { _ in
                    let result = height
                    if data.id == chips.last?.id {
                        height = 0
                    }
                    return result
                }
        }
    }
}

#Preview("Tap to expand") {
    StatefulPreviewWrapper(false) { isExpanded in
        ChipContainerView(
            viewModel: MockFeatureIndicatorsViewModel(),
            tunnelState: .connected(
                .init(
                    entry: nil,
                    exit: .init(
                        endpoint: .init(
                            ipv4Relay: .init(ip: .allHostsGroup, port: 1234),
                            ipv4Gateway: .allHostsGroup,
                            ipv6Gateway: .broadcast,
                            publicKey: Data()
                        ),
                        hostname: "hostname",
                        location: .init(
                            country: "Sweden",
                            countryCode: "SE",
                            city: "Gothenburg",
                            cityCode: "gbg",
                            latitude: 1234,
                            longitude: 1234
                        )
                    ),
                    retryAttempt: 0
                ),
                isPostQuantum: false,
                isDaita: false
            ),
            isExpanded: isExpanded
        )
        .background(UIColor.secondaryColor.color)
    }
}
