import { shell } from 'electron';
import { connect } from 'react-redux';
import { RouteComponentProps, withRouter } from 'react-router';
import { bindActionCreators } from 'redux';
import Support from '../components/Support';
import { collectProblemReport, sendProblemReport } from '../lib/problem-report';
import { IReduxState, ReduxDispatch } from '../redux/store';
import supportActions from '../redux/support/actions';

const mapStateToProps = (state: IReduxState) => ({
  defaultEmail: state.support.email,
  defaultMessage: state.support.message,
  accountHistory: state.account.accountHistory,
  isOffline: state.connection.isBlocked,
  outdatedVersion: state.version.suggestedUpgrade ? true : false,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, props: RouteComponentProps) => {
  const { saveReportForm, clearReportForm } = bindActionCreators(supportActions, dispatch);

  return {
    onClose() {
      props.history.goBack();
    },
    viewLog(path: string) {
      shell.openItem(path);
    },
    saveReportForm,
    clearReportForm,
    collectProblemReport,
    sendProblemReport,
    onExternalLink: (url: string) => shell.openExternal(url),
  };
};

export default withRouter(connect(mapStateToProps, mapDispatchToProps)(Support));
