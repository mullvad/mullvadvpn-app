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

    @State var chipHeight: CGFloat = 0
    @State var fullContainerHeight: CGFloat = 0
    @State var visibleContainerHeight: CGFloat = 0

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width
            let chipsOverflow = !viewModel.isExpanded && (fullContainerHeight > chipHeight)
            let numberOfChips = chipsOverflow ? 2 : viewModel.chips.count

            HStack {
                ZStack(alignment: .topLeading) {
                    createChipViews(chips: Array(viewModel.chips.prefix(numberOfChips)), containerWidth: containerWidth)
                }
                .sizeOfView { visibleContainerHeight = $0.height }

                if chipsOverflow {
                    Text(LocalizedStringKey("\(viewModel.chips.count - numberOfChips) more..."))
                        .font(.subheadline)
                        .lineLimit(1)
                        .foregroundStyle(UIColor.primaryTextColor.color)
                        .padding(.bottom, 12)
                }

                Spacer()
            }
            .background(preRenderViewSize(containerWidth: containerWidth))
        }.frame(height: visibleContainerHeight)
    }

    // Renders all chips on screen, in this case specifically to get their combined height.
    // Used to determine if content would overflow if view was not expanded and should
    // only be called from a background modifier.
    private func preRenderViewSize(containerWidth: CGFloat) -> some View {
        ZStack(alignment: .topLeading) {
            createChipViews(chips: viewModel.chips, containerWidth: containerWidth)
        }
        .hidden()
        .sizeOfView { fullContainerHeight = $0.height }
    }

    private func createChipViews(chips: [ChipModel], containerWidth: CGFloat) -> some View {
        var width = CGFloat.zero
        var height = CGFloat.zero

        return ForEach(chips) { data in
            ChipView(item: data)
                .padding(EdgeInsets(top: 6, leading: 0, bottom: 6, trailing: 8))
                .alignmentGuide(.leading) { dimension in
                    if abs(width - dimension.width) > containerWidth {
                        width = 0
                        height -= dimension.height
                    }
                    let result = width
                    if data.id == chips.last!.id {
                        width = 0
                    } else {
                        width -= dimension.width
                    }
                    return result
                }
                .alignmentGuide(.top) { _ in
                    let result = height
                    if data.id == chips.last!.id {
                        height = 0
                    }
                    return result
                }
                .sizeOfView { chipHeight = $0.height }
        }
    }
}

#Preview("Normal") {
    ChipContainerView(viewModel: MockFeatureIndicatorsViewModel())
        .background(UIColor.secondaryColor.color)
}

#Preview("Expanded") {
    ChipContainerView(viewModel: MockFeatureIndicatorsViewModel(isExpanded: true))
        .background(UIColor.secondaryColor.color)
}
