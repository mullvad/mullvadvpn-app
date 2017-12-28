// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Preferences from '../components/Preferences';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => ({
  allowLan: state.settings.allowLan
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { backend } = props;
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  return {
    onClose: () => pushHistory('/settings'),
    onChangeAllowLan: (allowLan) => {
      backend.setAllowLan(allowLan);
    },
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Preferences);
