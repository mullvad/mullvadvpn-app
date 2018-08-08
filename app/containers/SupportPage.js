// @flow
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'connected-react-router';
import Support from '../components/Support';
import { openItem } from '../lib/platform';
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
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);

  return {
    onClose: () => pushHistory('/settings'),
    viewLog: (path) => openItem(path),
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
