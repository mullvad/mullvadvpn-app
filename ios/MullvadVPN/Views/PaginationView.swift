//
//  Untitled.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct PaginationView: View {
    let pages: [AnyView]
    @Binding var currentPage: Int

    init(pages: [any View], currentPage: Binding<Int>) {
        self.pages = pages.map { AnyView($0) }
        self._currentPage = currentPage
    }

    var body: some View {
        TabView(selection: $currentPage) {
            ForEach(pages.indices, id: \.self) { index in
                pages[index]
                    .tag(index)
            }
        }
        .tabViewStyle(.page)
    }
}
