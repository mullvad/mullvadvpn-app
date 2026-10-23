//// This Source Code Form is subject to the terms of the GPLv3 License.
//// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
////
//// This file incorporates work covered by the following copyright and
//// permission notice:
////
////   Copyright (c) Mullvad VPN AB. All rights reserved.
////
//// SPDX-License-Identifier: GPL-3.0-only
//
//import MullvadMockData
//import MullvadSettings
//import MullvadTypes
//import SwiftUI
//
//#Preview {
//    EditAccessMethodView(
//        viewModel: MockEditAccessMethodViewModel(isEnabled: true, isEditable: true)
//    )
//    .environment(\.locale, Locale(identifier: "de"))
//    .environment(\.dynamicTypeSize, .large)
//}
//
//@Observable
//private final class MockEditAccessMethodViewModel: EditAccessMethodViewModelProtocol {
//    let accessMethod: PersistentAccessMethod = .init(
//        id: UUID(), name: "Mock", isEnabled: true, proxyConfiguration: .direct)
//
//    let name: String = "Mock Access Method"
//    var task: Task<Void, Never>?
//
//    let infoHeaderConfig: InfoHeaderConfig? = InfoHeaderConfig(
//        body: NSLocalizedString("The app communicates with a Mullvad API server directly.", comment: ""),
//        link: String(format: NSLocalizedString("About %@ method...", comment: ""), "Direct")
//    )
//
//    let infoModalConfig: InfoModalConfig? = InfoModalConfig(
//        header: "Direct",
//        preamble: NSLocalizedString("The app communicates with a Mullvad API server directly.", comment: ""),
//        body: [
//            String(
//                format: NSLocalizedString(
//                    "With the “%@” method, the app communicates with a Mullvad API "
//                        + "server directly without any intermediate proxies.",
//                    comment: ""
//                ),
//                "Direct"
//            ),
//            NSLocalizedString("This can be useful when you are not affected by censorship.", comment: ""),
//        ]
//    )
//
//    var isEnabled: Bool = true
//
//    var isEditable: Bool = true
//
//    var testState: ApiAccessMethodTestState = .idle
//
//    init(
//        isEnabled: Bool = true,
//        isEditable: Bool = true
//    ) {
//        self.isEnabled = isEnabled
//        self.isEditable = isEditable
//    }
//
//    func startTesting() {
//        testState = .testing
//        task = Task {
//            try? await Task.sleep(for: .seconds(2))
//            guard !Task.isCancelled else { return }
//            self.testState =
//                [
//                    .reachable,
//                    .unreachable,
//                ].randomElement() ?? .reachable
//        }
//    }
//
//    func stopTesting() {
//        task?.cancel()
//        testState = .idle
//    }
//
//    func delete() {}
//}
