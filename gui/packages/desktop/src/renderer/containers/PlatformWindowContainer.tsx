import { connect } from 'react-redux';
import PlatformWindow from '../components/PlatformWindow';

import { ReduxState } from '../redux/store';

const mapStateToProps = (state: ReduxState) => ({
  arrowPosition: state.userInterface.arrowPosition,
});

export default connect(mapStateToProps)(PlatformWindow);
