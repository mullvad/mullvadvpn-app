import SwiftUI

struct EntryLocationView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    let onShowsTopOfTheListChange: (Bool) -> Void
    var body: some View {
        if viewModel.showDAITAInfo {
            DaitaWarningView {
                viewModel.showDaitaSettings()
            }
        } else {
            ExitLocationView(
                viewModel: viewModel, context: $viewModel.entryContext,
                onShowsTopOfTheListChange: onShowsTopOfTheListChange)
        }
    }
}
