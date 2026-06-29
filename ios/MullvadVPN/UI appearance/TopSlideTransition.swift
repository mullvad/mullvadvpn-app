//
//  TopSlideTransition.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-06-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
