// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

// This should probably live somewhere more central than `View controllers/Settings/SwiftUI components`.  Where exactly is to be determined.

import SwiftUI

/** A wrapper for providing a state binding for SwiftUI Views in #Preview. This takes as arguments an initial value for the binding and a block which accepts the binding and returns a View to be previewed
  The usage looks like:

 ```
 #Preview {
    StatefulPreviewWrapper(initvalue) { ComponentToBePreviewed(binding: $0) }
 }
 ```
  */

struct StatefulPreviewWrapper<Value, Content: View>: View {
    @State var value: Value
    var content: (Binding<Value>) -> Content

    var body: some View {
        content($value)
    }

    init(_ value: Value, content: @escaping (Binding<Value>) -> Content) {
        self._value = State(wrappedValue: value)
        self.content = content
    }
}
