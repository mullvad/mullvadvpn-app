import { connect } from 'react-redux';
import PlatformWindow from '../components/PlatformWindow';

import { IReduxState } from '../redux/store';

const mapStateToProps = (state: IReduxState) => ({
  arrowPosition: state.userInterface.arrowPosition,
});

export default connect(mapStateToProps)(PlatformWindow);
