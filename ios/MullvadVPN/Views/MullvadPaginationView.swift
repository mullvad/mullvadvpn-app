//
//  MullvadPaginationView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MullvadPaginationView: View {
    let pages: [AnyView]
    @Binding var currentPage: Int

    init(pages: [any View], currentPage: Binding<Int>) {
        self.pages = pages.map { AnyView($0) }
        self._currentPage = currentPage
    }

    var body: some View {
        VStack(spacing: 8) {
            TabView(selection: $currentPage) {
                ForEach(pages.indices, id: \.self) { index in
                    pages[index]
                        .tag(index)
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .never))

            if pages.count > 1 {
                HStack(spacing: 8) {
                    ForEach(pages.indices, id: \.self) { index in
                        Circle()
                            .fill(currentPage == index ? Color.mullvadTextPrimary : Color.mullvadTextSecondary)
                            .frame(width: 8, height: 8)
                    }
                }
            }
        }
        .padding(.bottom, 8)
    }
}
