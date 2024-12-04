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

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width

            var chipsToAdd = viewModel.chips
            var showMoreButton = false

            if !isExpanded {
                (chipsToAdd, showMoreButton) = viewModel.chipsToAdd(forContainerWidth: containerWidth)
            }

            return HStack {
                ZStack(alignment: .topLeading) {
                    if isExpanded {
                        createChipViews(chips: viewModel.chips, containerWidth: containerWidth)
                    } else {
                        createChipViews(chips: chipsToAdd, containerWidth: containerWidth)
                    }
                }

                if showMoreButton {
                    Text(LocalizedStringKey("\(viewModel.chips.count - chipsToAdd.count) more..."))
                        .font(.subheadline)
                        .lineLimit(1)
                        .foregroundStyle(UIColor.primaryTextColor.color)
                }

                Spacer()
            }
            .sizeOfView { chipContainerHeight = $0.height }
        }
        .frame(height: chipContainerHeight)
        .padding(.vertical, -5)
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

#Preview("Normal") {
    ChipContainerView(
        viewModel: MockFeatureIndicatorsViewModel(),
        isExpanded: .constant(false)
    )
    .background(UIColor.secondaryColor.color)
}

#Preview("Expanded") {
    ChipContainerView(
        viewModel: MockFeatureIndicatorsViewModel(),
        isExpanded: .constant(true)
    )
    .background(UIColor.secondaryColor.color)
}
