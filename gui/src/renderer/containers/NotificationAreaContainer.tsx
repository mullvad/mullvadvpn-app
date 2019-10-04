import { connect } from 'react-redux';

import { shell } from 'electron';
import { links } from '../../config.json';
import NotificationArea from '../components/NotificationArea';
import AccountExpiry from '../lib/account-expiry';
import withAppContext, { IAppReduxContext } from '../redux/context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState, _props: IAppReduxContext) => ({
  accountExpiry: state.account.expiry
    ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
    : undefined,
  tunnelState: state.connection.status,
  version: state.version,
  blockWhenDisconnected: state.settings.blockWhenDisconnected,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, _props: IAppReduxContext) => {
  return {
    onOpenDownloadLink() {
      shell.openExternal(links.download);
    },
    onOpenBuyMoreLink() {
      shell.openExternal(links.purchase);
    },
  };
};

export default withAppContext(
  connect(
    mapStateToProps,
    mapDispatchToProps,
  )(NotificationArea),
);
