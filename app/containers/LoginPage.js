// @flow

import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { push } from 'react-router-redux';
import Login from '../components/Login';
import accountActions from '../redux/account/actions';
import { links } from '../config';
import { openLink } from '../lib/platform';

import type { ReduxState, ReduxDispatch } from '../redux/store';
import type { SharedRouteProps } from '../routes';

const mapStateToProps = (state: ReduxState) => state;
const mapDispatchToProps = (dispatch: ReduxDispatch, props: SharedRouteProps) => {
  const { push: pushHistory } = bindActionCreators({ push }, dispatch);
  const { login, resetLoginError, updateAccountToken } = bindActionCreators(accountActions, dispatch);
  const { backend } = props;
  return {
    onSettings: () => {
      pushHistory('/settings');
    },
    onLogin: (account) => {
      login(backend, account);
    },
    onFirstChangeAfterFailure: () => {
      resetLoginError();
    },
    onExternalLink: (type) => openLink(links[type]),
    onAccountTokenChange: (token) => {
      updateAccountToken(token);
    },
    onRemoveAccountTokenFromHistory: (token) => backend.removeAccountFromHistory(token),
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Login);
