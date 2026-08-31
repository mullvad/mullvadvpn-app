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
