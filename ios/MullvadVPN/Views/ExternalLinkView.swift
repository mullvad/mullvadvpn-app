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

struct ExternalLinkView: View {
    let url: URL
    let label: String
    let font: Font
    let color: Color

    init(url: URL, label: String, font: Font, color: Color = .white) {
        self.url = url
        self.label = label
        self.font = font
        self.color = color
    }

    var body: some View {
        HStack(alignment: .center, spacing: 2) {
            Link(label, destination: url)
                .font(font)
                .underline()
            Image(.iconExtlink)
                .renderingMode(.original)
        }
        .tint(.white)
    }
}

#Preview {
    ExternalLinkView(
        url: URL(string: "http://www.mullvad.net")!,
        label: NSLocalizedString("Mullvad website", comment: ""),
        font: .mullvadTiny,
        color: Color.red
    )
    .background(Color.mullvadBackground)
}
