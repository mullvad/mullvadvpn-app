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

    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        GeometryReader { geo in
            let containerWidth = geo.size.width
            ZStack(alignment: .topLeading) {
                var width = CGFloat.zero
                var height = CGFloat.zero

                ForEach(viewModel.chips) { data in
                    ChipView(item: data)
                        .padding(5)
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
