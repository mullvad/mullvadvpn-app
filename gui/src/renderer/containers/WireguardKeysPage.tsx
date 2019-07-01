import { goBack, push } from 'connected-react-router';
import { shell } from 'electron';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { links } from '../../config.json';
import WireguardKeys from '../components/WireguardKeys';
import { IReduxState, ReduxDispatch } from '../redux/store';
import { ISharedRouteProps } from '../routes';

const mapStateToProps = (state: IReduxState, _props: ISharedRouteProps) => ({
  keyState: state.settings.wireguardKeyState,
  isOffline: state.connection.isBlocked,
});
const mapDispatchToProps = (dispatch: ReduxDispatch, props: ISharedRouteProps) => {
  const history = bindActionCreators({ push, goBack }, dispatch);
  return {
    onGenerateKey: () => props.app.generateWireguardKey(),
    onVerifyKey: (publicKey: string) => props.app.verifyWireguardKey(publicKey),
    onVisitWebsiteKey: () => shell.openExternal(links.manageKeys),
    onClose: () => history.goBack(),
  };
};

export default connect(
  mapStateToProps,
  mapDispatchToProps,
)(WireguardKeys);
