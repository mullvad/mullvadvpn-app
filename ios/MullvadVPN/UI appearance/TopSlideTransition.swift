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

// this is necessart because .move(edge. .top) propagates down to subsidiary views and
// contaminates their own transitions.

struct TopSlideTransition: Transition {
    func body(content: Content, phase: TransitionPhase) -> some View {
        let progress = phase.isIdentity ? 1.0 : 0.0
        content
            .visualEffect { content, proxy in
                content.offset(x: 0.0, y: -(proxy.size.height * (1 - progress)))
            }
    }
}

extension AnyTransition {
    public static var topSlide: AnyTransition { .init(TopSlideTransition()) }
}
