//
//  SettingsInfoView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
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

    // Extra spacing to allow for some room around the page indicators.
    var pageIndicatorSpacing: CGFloat {
        viewModel.pages.count > 1 ? 72 : 24
    }

    var body: some View {
        ZStack {
            TabView {
                contentView()
            }
            .padding(UIMetrics.SettingsInfoView.layoutMargins)
            .tabViewStyle(.page)
            .foregroundColor(Color(.primaryTextColor))
            .background {
                Color(.secondaryColor)
            }
            hiddenViewToStretchHeightInsideScrollView()
        }
    }

//    A TabView inside a Scrollview has no height. This hidden view stretches the TabView to have the size of the heighest page.
    private func hiddenViewToStretchHeightInsideScrollView() -> some View {
        return ZStack {
            contentView()
        }
        .padding(UIMetrics.SettingsInfoView.layoutMargins)
        .padding(.bottom, 1)
        .hidden()
    }

    private func bodyText(_ page: SettingsInfoViewModelPage) -> some View {
        (try? AttributedString(
            markdown: page.body,
            options: AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        )).map(Text.init) ?? Text(page.body)
    }

    private func contentView() -> some View {
        ForEach(viewModel.pages, id: \.self) { page in
            VStack {
                VStack(alignment: .leading, spacing: 16) {
                    Image(page.image)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                    bodyText(page)
                        .font(.subheadline)
                        .opacity(0.6)
                }
                Spacer()
            }
            .padding(.bottom, pageIndicatorSpacing)
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

#Preview("Single inside Scrollview") {
    ScrollView {
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
}

#Preview("Multiple inside Scrollview") {
    ScrollView {
        SettingsInfoView(viewModel: SettingsInfoViewModel(
            pages: [
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "SETTINGS_INFO_DAITA_PAGE_1",
                        tableName: "Settings",
                        value: """
                        **Attention: This increases network traffic and will also  negatively affect speed, latency, \
                        and battery usage. Use with caution on limited plans.**

                        DAITA (Defense against AI-guided Traffic Analysis) hides patterns in \
                        your encrypted VPN traffic.

                        By using sophisticated AI it’s possible to analyze the traffic of data \
                        packets going in and out of your device (even if the traffic is encrypted).
                        """,
                        comment: ""
                    ),
                    image: .daitaOffIllustration
                ),
                SettingsInfoViewModelPage(
                    body: NSLocalizedString(
                        "SETTINGS_INFO_DAITA_PAGE_2",
                        tableName: "Settings",
                        value: """
                        If an observer monitors these data packets, DAITA makes it significantly \
                        harder for them to identify which websites you are visiting or with whom \
                        you are communicating.

                        DAITA does this by carefully adding network noise and making all network \
                        packets the same size.

                        Not all our servers are DAITA-enabled. Therefore, we use multihop \
                        automatically to enable DAITA with any server.
                        """,
                        comment: ""
                    ),
                    image: .daitaOnIllustration
                ),
            ]
        ))
    }
}
