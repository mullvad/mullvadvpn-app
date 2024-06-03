//
//  SettingsView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-05-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI
import UIKit

// enum SettingsSection: String, Identifiable {
//     case main
//     case version
//     case problemReport
//
//     var id: Self {
//         self
//     }
// }

struct SettingsSection: Identifiable {
    let id = UUID()
    let items: [SettingsItem]
}

struct SettingsViewModel {
    let sections: [SettingsSection]
}

struct SettingsRowViewModel: Identifiable {
    let id = UUID()
    let item: SettingsItem
}

enum SettingsItem: String, CaseIterable, Identifiable {
    case vpnSettings
    case version
    case problemReport
    case faq
    case apiAccess

    var id: Self {
        self
    }

    var title: String {
        switch self {
        case .vpnSettings:
            return "VPN settings"
        case .version:
            return "App version"
        case .problemReport:
            return "Report a problem"
        case .faq:
            return "FAQs & Guides"
        case .apiAccess:
            return "API access"
        }
    }

    var accessibilityIdentifier: AccessibilityIdentifier {
        switch self {
        case .vpnSettings:
            return .vpnSettingsCell
        case .version:
            return .versionCell
        case .problemReport:
            return .problemReportCell
        case .faq:
            return .faqCell
        case .apiAccess:
            return .apiAccessCell
        }
    }
}

struct SettingsView: View {
    var viewModel: SettingsViewModel

    init(viewModel: SettingsViewModel) {
        self.viewModel = viewModel
        UITableView.appearance().backgroundColor = .clear
    }

    var body: some View {
        ZStack {
            Color(.secondaryColor).ignoresSafeArea()
            List {
                ForEach(viewModel.sections) { section in
                    Section {
                        ForEach(section.items) { item in
                            SettingsRow(viewModel: SettingsRowViewModel(item: item))
                                .background(
                                    NavigationLink(destination: Text("Hello, World!")) {}
                                        .opacity(0)
                                )
                                .listRowSeparatorTint(Color(UIColor.separator))
                                .listRowBackground(Color(UIColor.primaryColor))
                                .listRowInsets(EdgeInsets(
                                    top: 0,
                                    leading: UIMetrics.TableView.cellIndentationWidth,
                                    bottom: 0,
                                    trailing: UIMetrics.TableView.cellIndentationWidth)
                                )
                                .frame(height: 54)
                        }
                    }
//                    } header: {
////                        Color.red
////                            .listRowInsets(.init())
////                            .frame(height: 1)
//                        Text("")
////                            .font(.callout)
////                            .listRowInsets(.init())
//                    }
                }
            }
            .listStyle(.plain)
            .navigationTitle("Settings")
            .environment(\.defaultMinListHeaderHeight, 1)
        }
    }
}

struct SettingsRow: View {
    let viewModel: SettingsRowViewModel

    var body: some View {
        HStack {
            Text(viewModel.item.title)
                .foregroundStyle(.white)
            Spacer()
            Image(systemName: "chevron.right")
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 7)
                .foregroundStyle(.white)
        }
    }
}

struct SettingsView_Previews: PreviewProvider {
    static var previews: some View {
        SettingsView(viewModel: SettingsViewModel(sections: [
            SettingsSection(items: [.vpnSettings, .apiAccess]),
            SettingsSection(items: [.problemReport])
        ]))
    }
}

struct SettingsRow_Previews: PreviewProvider {
    static var previews: some View {
        SettingsRow(viewModel: SettingsRowViewModel(item: .apiAccess))
            .background(Color(UIColor.primaryColor))
    }
}

private struct CustomTextFieldStyle: TextFieldStyle {
    func _body(configuration: TextField<Self._Label>) -> some View {
        configuration
            .padding(EdgeInsets(top: 6, leading: 8, bottom: 6, trailing: 8))
            .background(Color.white)
            .cornerRadius(4)
            .font(.system(size: 15))
    }
}
