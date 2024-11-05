//
//  StatefulPreviewWrapper.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

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
