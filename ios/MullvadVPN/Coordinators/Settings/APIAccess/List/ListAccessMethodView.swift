//
//  ListAccessMethodViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import SwiftUI

protocol ListAccessViewModel: ObservableObject {
    var items: [ListAccessMethodItem] { get }
    var itemInUse: ListAccessMethodItem? { get }
    func addNewMethod()
    func methodSelected(_ method: ListAccessMethodItem)
    func showAbout()
}

struct ListAccessMethodView<ViewModel>: View where ViewModel: ListAccessViewModel {
    @ObservedObject var viewModel: ViewModel

    init(viewModel: ViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            let text = NSLocalizedString(
                "Manage and add custom methods to access the Mullvad API.",
                comment: ""
            )
            let about = NSLocalizedString("About API access…", comment: "")

            MullvadList(
                viewModel.items,
                header: {
                    MullvadInfoHeaderView(
                        bodyText: "\(text) ",
                        link: about,
                        onTapLink: viewModel.showAbout
                    )
                },
                footer: {
                    MainButton(
                        text: LocalizedStringKey("Add"),
                        style: .default
                    ) {
                        viewModel.addNewMethod()
                    }
                    .accessibilityIdentifier(AccessibilityIdentifier.addAccessMethodButton.asString)
                },
                content: { item in
                    let accessibilityId: AccessibilityIdentifier? =
                        switch item.id {
                        case AccessMethodRepository.directId:
                            AccessibilityIdentifier.accessMethodDirectCell
                        case AccessMethodRepository.bridgeId:
                            AccessibilityIdentifier.accessMethodBridgesCell
                        case AccessMethodRepository.encryptedDNSId:
                            AccessibilityIdentifier.accessMethodEncryptedDNSCell
                        default:
                            nil
                        }
                    let state =
                        viewModel.itemInUse?.id == item.id
                        ? NSLocalizedString("In use", comment: "")
                        : (!item.isEnabled
                            ? NSLocalizedString("Off", comment: "")
                            : nil)
                    MullvadListNavigationItemView(
                        item: MullvadListNavigationItem(
                            id: item.id,
                            title: item.name,
                            state: state,
                            detail: item.detail,
                            accessibilityIdentifier: accessibilityId
                        ) {
                            viewModel.methodSelected(item)
                        }
                    )
                }
            )
            .accessibilityIdentifier(
                AccessibilityIdentifier.apiAccessListView.asString
            )
            .apply {
                if #available(iOS 16.4, *) {
                    $0.scrollBounceBehavior(.basedOnSize)
                } else {
                    $0
                }
            }
            Spacer()
        }
        .background(Color.mullvadBackground)
    }
}

#Preview {
    NavigationView {
        ListAccessMethodView(
            viewModel: ListAccessViewModelBridge(
                interactor: ListAccessMethodInteractor(
                    repository: AccessMethodRepository()
                ), delegate: nil
            )
        )
        .navigationTitle("API Access")
    }
}
