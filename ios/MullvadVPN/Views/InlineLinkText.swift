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

struct InlineLinkText: View {
    private let text: String
    private let linkTitle: String
    private let font: Font
    private let textColor: Color
    private let linkColor: Color
    private let onLinkTapped: (() -> Void)?

    init(
        text: String,
        linkTitle: String,
        font: Font = .body,
        textColor: Color = .mullvadTextSecondary,
        linkColor: Color = .mullvadTextPrimary,
        onLinkTapped: (() -> Void)?
    ) {
        self.text = text
        self.linkTitle = linkTitle
        self.font = font
        self.textColor = textColor
        self.linkColor = linkColor
        self.onLinkTapped = onLinkTapped
    }

    var body: some View {
        Text(makeAttributedString())
            .environment(
                \.openURL,
                OpenURLAction { _ in
                    onLinkTapped?()
                    return .handled
                })
    }

    private func makeAttributedString() -> AttributedString {
        var result = AttributedString(text)
        result.foregroundColor = textColor

        var link = AttributedString(" \(linkTitle)")
        link.foregroundColor = linkColor
        link.font = .mullvadTiny
        link.link = URL(string: "app://inline-link")!

        result.append(link)

        return result
    }
}

#Preview("Inline Link in Navigation Header") {
    InlineLinkTextPreview()
}
private struct InlineLinkTextPreview: View {
    @State private var showDetails = false

    var body: some View {
        InlineLinkText(
            text: "The app communicates with a Mullvad API server directly.",
            linkTitle: "About \("Direct") method..."
        ) {
            showDetails = true
        }
        .background(Color.mullvadBackground)
        .sheet(isPresented: $showDetails) {
            Text("Direct Method")
                .navigationTitle("Direct Method")
        }
    }
}
