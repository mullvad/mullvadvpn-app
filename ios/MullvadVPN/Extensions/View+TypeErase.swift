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

extension View {
    /// Type-erases an optional view to `AnyView?`, collapsing both an omitted view of type `EmptyView`
    /// and a conditional builder that produced no content down to `nil`. This keeps empty views from
    /// rendering an otherwise sized, background-filled container.
    func typeErase() -> AnyView? {
        if let optional = self as? AnyOptionalView {
            return optional.typeErased
        }
        return self is EmptyView ? nil : AnyView(self)
    }
}

/// Lets `View.typeErase(_:)` recognise an `Optional` view (produced by a conditional view builder)
/// at runtime and map its `.none` case to `nil` instead of a non-empty `AnyView`.
private protocol AnyOptionalView {
    var typeErased: AnyView? { get }
}

extension Optional: AnyOptionalView where Wrapped: View {
    var typeErased: AnyView? { map { AnyView($0) } }
}
