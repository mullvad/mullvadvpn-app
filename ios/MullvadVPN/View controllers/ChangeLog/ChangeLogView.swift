//
//  ChangeLogView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-01-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct ChangeLogView<ViewModel>: View where ViewModel: ChangeLogViewModelProtocol {
    @ObservedObject var viewModel: ViewModel

    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        ZStack {
            UIColor.secondaryColor.color.ignoresSafeArea()
            VStack {
                Text(viewModel.changeLog?.title ?? "")
                    .font(.title)
                    .fontWeight(.semibold)
                    .foregroundColor(UIColor.primaryTextColor.color)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, UIMetrics.contentInsets.left)
                    .padding(.top, UIMetrics.TableView.emptyHeaderHeight)
                List {
                    ForEach(viewModel.changeLog?.changes ?? [], id: \.self) { item in
                        BulletPointText(text: item)
                            .listRowSeparator(.hidden)
                            .listRowBackground(Color.clear)
                    }
                    .padding(.horizontal, UIMetrics.contentInsets.left)
                }
                .listStyle(.plain)
                .frame(maxHeight: .infinity)
            }
        }
        .onAppear {
            viewModel.getLatestChanges()
        }
    }
}

#Preview {
    ChangeLogView(viewModel: MockChangeLogViewModel())
}
