import { push } from 'connected-react-router';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Launch from '../components/Launch';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (_state: IReduxState) => ({});
const mapDispatchToProps = (dispatch: ReduxDispatch) => {
  const history = bindActionCreators({ push }, dispatch);
  return {
    openSettings() {
      history.push('/settings');
    },
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Launch);
