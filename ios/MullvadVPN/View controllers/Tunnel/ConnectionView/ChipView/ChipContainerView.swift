//
//  ChipContainerView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ChipContainerView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel
    @Binding var isExpanded: Bool

    @State private var chipContainerHeight: CGFloat = .zero
    private let verticalPadding: CGFloat = 6

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
                    isExpanded.toggle()
                }
                .font(.subheadline)
                .lineLimit(1)
                .foregroundStyle(UIColor.primaryTextColor.color)
                .showIf(showMoreButton)

                Spacer()
            }
            .sizeOfView { size in
                withAnimation {
                    chipContainerHeight = size.height
                }
            }
        }
        .frame(height: chipContainerHeight)
        .padding(.vertical, -(verticalPadding - 1)) // Remove extra padding from chip views on top and bottom.
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
            isExpanded: isExpanded
        )
        .background(UIColor.secondaryColor.color)
    }
}
