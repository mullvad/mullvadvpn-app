//
//  ActivityIndicator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct CustomProgressView: View {
    var style: Style
    @State private var angle: Double = 0
    @ObservedObject var connectionViewModel: ConnectionViewViewModel

    var body: some View {
        Image(.iconSpinner)
            .resizable()
            .frame(width: style.size.width, height: style.size.height)
            .rotationEffect(.degrees(angle))
            .onAppear {
                withAnimation(Animation.linear(duration: 0.6).repeatForever(autoreverses: false)) {
                    angle = 360
                }
            }
            .onDisappear {
                withAnimation(Animation.linear(duration: 0.6).repeatForever(autoreverses: false)) {
                    angle = 0
                }
            }
//            .showIf(connectionViewModel.showsActivityIndicator)
    }
}

#Preview {
    ConnectionViewComponentPreview(showIndicators: true, isExpanded: true) { _, viewModel, _ in
        CustomProgressView(style: .large, connectionViewModel: viewModel)
    }
}

extension CustomProgressView {
    enum Style {
        case small, medium, large

        var size: CGSize {
            switch self {
            case .small:
                CGSize(width: 16, height: 16)
            case .medium:
                CGSize(width: 20, height: 20)
            case .large:
                CGSize(width: 60, height: 60)
            }
        }
    }
}
