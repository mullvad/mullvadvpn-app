//
//  ChipContainerView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-12-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SwiftUI
struct ChipContainerView<ViewModel>: View where ViewModel: ChipViewModelProtocol {
    @ObservedObject var viewModel: ViewModel

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width
            let chipsPerLine = 2
            if viewModel.isExpanded {
                ZStack(alignment: .topLeading) {
                    createChipViews(chips: viewModel.chips, containerWidth: containerWidth)
                }
            } else {
                HStack {
                    createChipViews(chips: Array(viewModel.chips.prefix(chipsPerLine)), containerWidth: containerWidth)
                    if viewModel.chips.count > chipsPerLine {
                        Text(LocalizedStringKey("\(viewModel.chips.count - chipsPerLine) more..."))
                            .font(.subheadline)
                            .lineLimit(1)
                            .foregroundStyle(UIColor.primaryTextColor.color)
                    }
                }
            }
        }
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
                    if data.id == viewModel.chips.last!.id {
                        width = 0
                    } else {
                        width -= dimension.width
                    }
                    return result
                }
                .alignmentGuide(.top) { _ in
                    let result = height
                    if data.id == viewModel.chips.last!.id {
                        height = 0
                    }
                    return result
                }
        }
    }
}

#Preview("Normal") {
    ChipContainerView(viewModel: FeaturesIndicatoresMockViewModel())
        .background(UIColor.secondaryColor.color)
}

#Preview("Expanded") {
    ChipContainerView(viewModel: FeaturesIndicatoresMockViewModel(isExpanded: true))
        .background(UIColor.secondaryColor.color)
}
