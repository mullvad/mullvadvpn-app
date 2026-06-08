//
//  ResizableImageView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct ResizableImageView: View {
    enum Layout {
        case square(CGFloat)
        case banner
    }

    @ScaledMetric private var baseSize: CGFloat = 48

    let image: Image
    let layout: Layout

    var body: some View {
        switch layout {
        case .square(let size):
            image
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(
                    width: scaled(size),
                    height: scaled(size)
                )

        case .banner:
            image
                .resizable()
                .aspectRatio(contentMode: .fit)
        }
    }

    private func scaled(_ value: CGFloat) -> CGFloat {
        value == 0 ? baseSize : value
    }
}
#Preview("ResizableBannerView", traits: .sizeThatFitsLayout) {
    VStack(spacing: 0) {
        ResizableImageView(image: .mullvadIconInfo, layout: .square(48))
        ResizableImageView(
            image: Image(.ianSolutionIllustration),
            layout: .banner)
    }
    .background(Color.gray.opacity(0.2))
}
