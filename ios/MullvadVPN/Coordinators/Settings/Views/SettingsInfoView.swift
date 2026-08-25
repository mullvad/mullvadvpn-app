// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct SettingsInfoPageView: View {
    let text: String?
    let image: ImageResource
    var customView: AnyView?

    init<Content: View>(
        text: String? = nil,
        image: ImageResource,
        @ViewBuilder customView: (() -> Content) = { EmptyView() }
    ) {
        self.text = text
        self.image = image
        self.customView = AnyView(customView())
    }

    var body: some View {
        VStack {
            VStack(alignment: .leading, spacing: 16) {
                Image(image)
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                if let text {
                    bodyText(text)
                        .fixedSize(horizontal: false, vertical: true)
                        .font(.mullvadTiny)
                        .foregroundStyle(Color.mullvadTextSecondary)
                }
                if let customView {
                    customView
                        .fixedSize(horizontal: false, vertical: true)
                }
            }
            Spacer()
        }
    }

    private func bodyText(_ text: String) -> (some View)? {
        (try? AttributedString(
            markdown: text,
            options: AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)
        )).map(Text.init) ?? Text(text)
    }
}

struct SettingsInfoView: View {
    var pages: [SettingsInfoPageView]

    // Extra spacing to allow for some room around the page indicators.
    var pageIndicatorSpacing: CGFloat {
        pages.count > 1 ? 48 : 0
    }

    init(@SettingsInfoViewModelPageBuilder pages: () -> [SettingsInfoPageView]) {
        self.pages = pages()
    }

    var body: some View {
        ZStack {
            TabView {
                contentView()
            }
            .tabViewStyle(.page)
            .foregroundColor(Color(.primaryTextColor))
            .background {
                Color(.secondaryColor)
            }
            hiddenViewToStretchHeightInsideScrollView()
        }
    }

    // A TabView inside a Scrollview has no height. This hidden view stretches the TabView to have the size
    // of the heighest page.
    private func hiddenViewToStretchHeightInsideScrollView() -> some View {
        return ZStack {
            contentView()
        }
        .padding(.bottom, 1)
        .hidden()
    }

    private func contentView() -> some View {
        ForEach(pages.indices, id: \.self) { index in
            pages[index]
        }
        .padding(.bottom, pageIndicatorSpacing)
        .padding(UIMetrics.SettingsInfoView.layoutMargins)
    }
}

// Makes the initializer of SettingsInfoView pretty
@resultBuilder
struct SettingsInfoViewModelPageBuilder {
    static func buildBlock(
        _ components: SettingsInfoPageView...
    ) -> [SettingsInfoPageView] {
        components
    }
}

#Preview("Single page") {
    SettingsInfoView {
        SettingsInfoPageView(
            text: """
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                """,
            image: .multihopIllustrationGeneral
        )
    }
}

#Preview("Multiple pages") {
    SettingsInfoView {
        SettingsInfoPageView(
            text: """
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                """,
            image: .multihopIllustrationGeneral
        )
        SettingsInfoPageView(
            text: """
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                Multihop routes your traffic into one WireGuard server and out another, making it \
                harder to trace. This results in increased latency but increases anonymity online.
                """,
            image: .multihopIllustrationWhenNeeded
        )
    }
}

#Preview("Single inside Scrollview") {
    ScrollView {
        SettingsInfoView {
            SettingsInfoPageView(
                text: """
                    Multihop routes your traffic into one WireGuard server and out another, making it \
                    harder to trace. This results in increased latency but increases anonymity online.
                    """,
                image: .multihopIllustrationGeneral
            )
        }
    }
}

#Preview("Multiple inside Scrollview") {
    ScrollView {
        SettingsInfoView {
            SettingsInfoPageView(
                text: NSLocalizedString(
                    """
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
            )
            SettingsInfoPageView(
                text: NSLocalizedString(
                    """
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
            )
        }
    }
}
