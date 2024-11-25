//
//  SettingsInfoView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SettingsInfoViewModel {
    let pages: [SettingsInfoViewModelPage]
}

struct SettingsInfoViewModelPage: Hashable {
    let body: String
    let image: ImageResource
}

struct SettingsInfoView: View {
    let viewModel: SettingsInfoViewModel
    @State var height: CGFloat = 0

    // Extra spacing to allow for some room around the page indicators.
    var pageIndicatorSpacing: CGFloat {
        viewModel.pages.count > 1 ? 72 : 24
    }

    var body: some View {
        TabView {
            ForEach(viewModel.pages, id: \.self) { page in
                VStack {
                    contentView(for: page)
                    Spacer()
                }
                .padding(UIMetrics.SettingsInfoView.layoutMargins)
            }
        }
        .frame(
            height: height + pageIndicatorSpacing
        )
        .tabViewStyle(.page)
        .foregroundColor(Color(.primaryTextColor))
        .background {
            Color(.secondaryColor)
            preRenderViewSize()
        }
    }

    private func contentView(for page: SettingsInfoViewModelPage) -> some View {
        VStack(alignment: .leading, spacing: 16) {
            Image(page.image)
                .resizable()
                .aspectRatio(contentMode: .fit)
            Text(page.body)
                .font(.subheadline)
                .opacity(0.6)
        }
    }

    // Renders the content of each page, determining the maximum height between them
    // when laid out on screen. Since we only want this to update the real view
    // this function should be called from a .background() and its contents hidden.
    private func preRenderViewSize() -> some View {
        ZStack {
            ForEach(viewModel.pages, id: \.self) { page in
                contentView(for: page)
            }
        }
        .hidden()
        .sizeOfView { size in
            if size.height > height {
                height = size.height
            }
        }
    }
}

#Preview("Single page") {
    SettingsInfoView(viewModel: SettingsInfoViewModel(
        pages: [
            SettingsInfoViewModelPage(
                body: """
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                """,
                image: .multihopIllustration
            ),
        ]
    ))
}

#Preview("Multiple pages") {
    SettingsInfoView(viewModel: SettingsInfoViewModel(
        pages: [
            SettingsInfoViewModelPage(
                body: """
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                """,
                image: .multihopIllustration
            ),
            SettingsInfoViewModelPage(
                body: """
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.

                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                """,
                image: .multihopIllustration
            ),
        ]
    ))
}
