import { RoutePath } from '../../../shared/routes';
import { LoginState } from '../../redux/account/reducers';

export function getNavigationBase(connectedToDaemon: boolean, loginState: LoginState): RoutePath {
  if (connectedToDaemon) {
    if (loginState.type === 'none' && loginState.deviceRevoked) {
      return RoutePath.deviceRevoked;
    } else if (
      loginState.type === 'too many devices' ||
      (loginState.type === 'failed' && loginState.error === 'too-many-devices')
    ) {
      return RoutePath.tooManyDevices;
    } else if (
      loginState.type === 'none' ||
      loginState.type === 'logging in' ||
      loginState.type === 'failed'
    ) {
      return RoutePath.login;
    } else if (loginState.type === 'ok' && loginState.expiredState === 'expired') {
      return RoutePath.expired;
    } else if (loginState.type === 'ok' && loginState.expiredState === 'time_added') {
      return RoutePath.timeAdded;
    } else {
      return RoutePath.main;
    }
  } else {
    return RoutePath.launch;
  }
}
