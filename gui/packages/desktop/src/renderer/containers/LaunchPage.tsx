import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { push } from 'connected-react-router';
import Launch from '../components/Launch';

import { ReduxState, ReduxDispatch } from '../redux/store';
import { SharedRouteProps } from '../routes';

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
