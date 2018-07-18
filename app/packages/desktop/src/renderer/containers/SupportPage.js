// @flow

import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { goBack } from 'connected-react-router';
import Support from '../components/Support';
import { collectProblemReport, sendProblemReport } from '../lib/problem-report';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';
import supportActions from '../redux/support/actions';

const mapStateToProps = (state: ReduxState) => ({
  defaultEmail: state.support.email,
  defaultMessage: state.support.message,
  accountHistory: state.account.accountHistory,
});

const mapDispatchToProps = (dispatch: ReduxDispatch, _props: SharedRouteProps) => {
  const { saveReportForm, clearReportForm } = bindActionCreators(supportActions, dispatch);
  const history = bindActionCreators({ goBack }, dispatch);

  return {
    onClose: () => history.goBack(),
    viewLog: (path) => shell.openItem(path),
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
