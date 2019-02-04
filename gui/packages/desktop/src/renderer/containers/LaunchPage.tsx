import { push } from 'connected-react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Launch from '../components/Launch';

import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (_state: IReduxState) => ({});
const mapDispatchToProps = (dispatch: ReduxDispatch, _props: ISharedRouteProps) => {
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
