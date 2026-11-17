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

enum SizeToFitPresentationDetent {
    case sizeToFit
}

private struct SizeToFitModifier: ViewModifier {
    let additionalDetents: Set<PresentationDetent>
    @State private var contentHeight = 0.0

    func body(content: Content) -> some View {
        content
            .onGeometryChange(for: CGFloat.self) {
                $0.size.height
            } action: { height in
                contentHeight = height
            }
            .presentationDetents(Set([.height(contentHeight)]).union(additionalDetents))
    }
}

extension View {
    func presentationDetents(
        _ detent: SizeToFitPresentationDetent,
        additional: Set<PresentationDetent> = []
    ) -> some View {
        modifier(SizeToFitModifier(additionalDetents: additional))
    }
}
