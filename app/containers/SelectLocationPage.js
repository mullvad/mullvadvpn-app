import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import SelectLocation from '../components/SelectLocation';
import settingsActions from '../actions/settings';

const mapStateToProps = (state) => state;
const mapDispatchToProps = (dispatch) => bindActionCreators(settingsActions, dispatch);

export default connect(mapStateToProps, mapDispatchToProps)(SelectLocation);
