//
//  DebugView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

@MainActor
struct DebugView<ViewModel: DebugViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var alert: MullvadAlert?

    @State private var showRelays: Bool = false
    @State private var showSettings: Bool = false

    var body: some View {
        ZStack {
            ScrollView {
                VStack(alignment: .leading) {
                    HStack {
                        Spacer()
                    }

                    connection()
                    settings()
                }
                .background(Color.mullvadBackground)
                Spacer()
            }
            .padding(EdgeInsets(UIMetrics.contentLayoutMargins))
        }
        .mullvadAlert(item: $alert)
        .font(.mullvadTiny)
        .foregroundStyle(Color.mullvadTextPrimary)
        .background(Color.mullvadBackground)

        Spacer()
    }
}

extension DebugView {
    private func connection() -> some View {
        VStack {
            Button {
                withAnimation {
                    showRelays.toggle()
                }
            } label: {
                Text("Connection".excludeLocalization)
                    .font(.mullvadMedium)
                    .foregroundStyle(Color.mullvadTextSecondary)
                Spacer()
            }

            RowSeparator()

            VStack(alignment: .leading) {
                ForEach(Array(viewModel.connection), id: \.title) { item in
                    createSection(title: item.title, rows: item.data)
                }
            }
            .showIf(showRelays)
        }
    }

    private func settings() -> some View {
        VStack {
            Button {
                withAnimation {
                    showSettings.toggle()
                }
            } label: {
                Text("Settings".excludeLocalization)
                    .font(.mullvadMedium)
                    .foregroundStyle(Color.mullvadTextSecondary)
                Spacer()
            }

            RowSeparator()

            VStack(alignment: .leading) {
                ForEach(Array(viewModel.settings), id: \.title) { item in
                    createSection(title: item.title, rows: item.data)
                }
            }.showIf(showSettings)
        }
    }

    private func createSection(title: String, rows: [String]) -> some View {
        VStack(alignment: .leading) {
            Text(title)
                .font(.mullvadSmallSemiBold)
                .foregroundStyle(Color.mullvadTextSecondary)
            ForEach(rows, id: \.self) {
                Text($0)
            }

            RowSeparator()
        }
    }
}

#Preview {
    DebugView(viewModel: MockDebugViewModel())
}
