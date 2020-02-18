import { connect } from 'react-redux';

import { shell } from 'electron';
import { links } from '../../config.json';
import AccountExpiry from '../../shared/account-expiry';
import NotificationArea from '../components/NotificationArea';
import withAppContext, { IAppContext } from '../context';
import { IReduxState, ReduxDispatch } from '../redux/store';

const mapStateToProps = (state: IReduxState, _props: IAppContext) => ({
  accountExpiry: state.account.expiry
    ? new AccountExpiry(state.account.expiry, state.userInterface.locale)
    : undefined,
  tunnelState: state.connection.status,
  version: state.version,
  blockWhenDisconnected: state.settings.blockWhenDisconnected,
});

const mapDispatchToProps = (_dispatch: ReduxDispatch, props: IAppContext) => {
  return {
    onOpenDownloadLink(): Promise<void> {
      return shell.openExternal(links.download);
    },
    onOpenBuyMoreLink(): Promise<void> {
      return props.app.openLinkWithAuth(links.purchase);
    },
  };
};

export default withAppContext(connect(mapStateToProps, mapDispatchToProps)(NotificationArea));
