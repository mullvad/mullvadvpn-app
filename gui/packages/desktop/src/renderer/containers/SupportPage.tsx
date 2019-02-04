import { goBack } from 'connected-react-router';
import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import Support from '../components/Support';
import { collectProblemReport, sendProblemReport } from '../lib/problem-report';

import { IReduxState, ReduxDispatch } from '../redux/store';
import supportActions from '../redux/support/actions';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState) => ({
  defaultEmail: state.support.email,
  defaultMessage: state.support.message,
  accountHistory: state.account.accountHistory,
  isOffline: state.connection.isBlocked,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, _props: ISharedRouteProps) => {
  const { saveReportForm, clearReportForm } = bindActionCreators(supportActions, dispatch);
  const history = bindActionCreators({ goBack }, dispatch);

  return {
    onClose: () => {
      history.goBack();
    },
    viewLog: (path: string) => shell.openItem(path),
    saveReportForm,
    clearReportForm,
    collectProblemReport,
    sendProblemReport,
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(Support);
