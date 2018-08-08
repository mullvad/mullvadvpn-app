// @flow

import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { push } from 'connected-react-router';
import Launch from '../components/Launch';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (_state: ReduxState) => ({});
const mapDispatchToProps = (dispatch: ReduxDispatch, _props: SharedRouteProps) => {
  const history = bindActionCreators({ push }, dispatch);
  return {
    openSettings: () => {
      history.push('/settings');
    },
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Launch);
