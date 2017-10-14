import { connect } from 'react-redux';
import { push } from 'react-router-redux';
import Support from '../components/Support';

const mapStateToProps = (state) => {
  return state;
};

const mapDispatchToProps = (dispatch, _props) => {
  return {
    onClose: () => dispatch(push('/settings')),
    onViewLogs: () => {},
    onSend: (_report) => {}
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Support);
