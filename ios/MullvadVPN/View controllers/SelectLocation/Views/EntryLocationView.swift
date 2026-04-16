import SwiftUI

struct EntryLocationView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    let onScrollVisibilityChange: (Bool) -> Void

    var body: some View {
        if viewModel.showMultihopInfo {
            MultihopWhenNeededInfoView(viewModel: viewModel)
        } else {
            ExitLocationView(
                viewModel: viewModel, context: $viewModel.entryContext,
                onScrollVisibilityChange: onScrollVisibilityChange)
        }
    }
}
