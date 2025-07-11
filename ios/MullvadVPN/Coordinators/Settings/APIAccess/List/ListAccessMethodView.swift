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
                "ACCESS_METHOD_HEADER_BODY",
                tableName: "APIAccess",
                value: "Manage default and setup custom methods to access the Mullvad API. ",
                comment: ""
            )
            let about = NSLocalizedString(
                "ACCESS_METHOD_HEADER_BODY",
                tableName: "APIAccess",
                value: "About API access…",
                comment: ""
            )

            MullvadList(viewModel.items, header: {
                MullvadInfoHeaderView(
                    bodyText: text,
                    link: about,
                    onTapLink: viewModel.showAbout
                )
                .padding(.bottom, 16)
            }, footer: {
                MainButton(
                    text: LocalizedStringKey("Add"),
                    style: .default
                ) {
                    viewModel.addNewMethod()
                }
                .padding(.top, 24)
            }, content: { item in
                let accessibilityId: AccessibilityIdentifier? = switch item.id {
                case AccessMethodRepository.directId:
                    AccessibilityIdentifier.accessMethodDirectCell
                case AccessMethodRepository.bridgeId:
                    AccessibilityIdentifier.accessMethodBridgesCell
                case AccessMethodRepository.encryptedDNSId:
                    AccessibilityIdentifier.accessMethodEncryptedDNSCell
                default:
                    nil
                }
                let state = viewModel.itemInUse?.id == item.id
                    ? NSLocalizedString(
                        "LIST_ACCESS_METHODS_IN_USE_ITEM",
                        tableName: "APIAccess",
                        value: "In use",
                        comment: ""
                    )
                    : (
                        !item.isEnabled
                            ? NSLocalizedString(
                                "LIST_ACCESS_METHODS_DISABLED",
                                tableName: "APIAccess",
                                value: "Disabled",
                                comment: ""
                            )
                            : nil
                    )
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
            })
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
            .accessibilityIdentifier(AccessibilityIdentifier.addAccessMethodButton.asString)
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
