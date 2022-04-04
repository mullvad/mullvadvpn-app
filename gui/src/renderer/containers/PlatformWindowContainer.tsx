import { connect } from 'react-redux';

import PlatformWindow from '../components/PlatformWindow';
import { IReduxState } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  arrowPosition: state.userInterface.arrowPosition,
  unpinnedWindow: state.settings.guiSettings.unpinnedWindow,
});

export default connect(mapStateToProps)(PlatformWindow);
