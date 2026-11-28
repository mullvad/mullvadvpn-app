import { RoutePath } from '../../../shared/routes';

export function getNavigationBase(connectedToDaemon: boolean, hasIdentity: boolean): RoutePath {
  if (connectedToDaemon) {
    return hasIdentity ? RoutePath.main : RoutePath.importConfig;
  } else {
    return RoutePath.launch;
  }
}
