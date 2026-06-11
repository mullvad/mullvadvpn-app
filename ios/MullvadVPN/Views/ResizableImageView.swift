//
//  ResizableImageView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct ResizableImageView: View {
    enum Dimension {
        case width(CGFloat)
        case height(CGFloat)
    }

    let image: Image
    let dimension: Dimension?

    @ScaledMetric(relativeTo: .body)
    private var dynamicScale = 1.0

    init(image: Image, dimension: Dimension? = nil) {
        self.image = image
        self.dimension = dimension
    }

    var body: some View {
        image
            .resizable()
            .aspectRatio(contentMode: .fit)
            .modifier(
                FrameModifier(
                    dimension: dimension,
                    scale: dynamicScale
                ))
    }
}

private struct FrameModifier: ViewModifier {
    let dimension: ResizableImageView.Dimension?
    let scale: CGFloat

    func body(content: Content) -> some View {
        switch dimension {
        case .width(let width):
            content.frame(width: width * scale)

        case .height(let height):
            content.frame(height: height * scale)
        case nil:
            content
                .aspectRatio(contentMode: .fit)
        }
    }
}

#Preview("ResizableBannerView", traits: .sizeThatFitsLayout) {
    VStack(spacing: 0) {
        ResizableImageView(image: .mullvadIconInfo, dimension: .width(48))
        ResizableImageView(
            image: Image(.ianSolutionIllustration),
            dimension: .width(.infinity))
        ResizableImageView(image: .mullvadIconInfo)
    }
    .background(Color.gray.opacity(0.2))
}
