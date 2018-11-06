// @flow

import { connect } from 'react-redux';
import PlatformWindow from '../components/PlatformWindow';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  arrowPosition: state.userInterface.arrowPosition,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, _props: SharedRouteProps) => ({});

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(PlatformWindow);
