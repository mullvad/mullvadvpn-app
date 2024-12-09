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
    @State private var isExpanded = false

    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width
            let chipsPerLine = 2
            if isExpanded {
                ZStack(alignment: .topLeading) {
                    createChipViews(chips: viewModel.chips, containerWidth: containerWidth)
                }
            } else {
                HStack {
                    createChipViews(chips: Array(viewModel.chips.prefix(chipsPerLine)), containerWidth: containerWidth)
                    if viewModel.chips.count > chipsPerLine {
                        Button(action: {
                            withAnimation {
                                isExpanded.toggle()
                            }
                        }, label: {
                            Text("Show More \(viewModel.chips.count - chipsPerLine)")
                                .font(.body)
                                .lineLimit(1)
                                .foregroundStyle(Color(uiColor: .primaryTextColor))
                        })
                    }
                    Spacer()
                }
            }
        }
    }

    private func createChipViews(chips: [ChipModel], containerWidth: CGFloat) -> some View {
        var width = CGFloat.zero
        var height = CGFloat.zero

        return ForEach(chips) { data in
            ChipView(item: data)
                .padding(UIMetrics.padding4)
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

#Preview("ChipContainerView") {
    ChipContainerView(viewModel: MockChipViewModel())
}

private class MockChipViewModel: ChipViewModelProtocol {
    @Published var chips: [ChipModel] = (5 ..< 20).map { index in
        let letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        return ChipModel(name: LocalizedStringKey(String((0 ..< index).map { _ in letters.randomElement()! })))
    }
}
