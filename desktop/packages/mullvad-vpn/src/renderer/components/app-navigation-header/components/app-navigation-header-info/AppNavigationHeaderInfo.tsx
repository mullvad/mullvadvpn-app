import { Info, type InfoProps } from '../../../info';
import { AppNavigationHeaderInfoButton } from './components/app-navigation-header-info-button';

export type AppNavigationHeaderInfoProps = InfoProps;

function AppNavigationHeaderInfo(props: AppNavigationHeaderInfoProps) {
  return <Info {...props} />;
}

const AppNavigationHeaderInfoNamespace = Object.assign(AppNavigationHeaderInfo, {
  Dialog: Info.Dialog,
  Button: AppNavigationHeaderInfoButton,
});

export { AppNavigationHeaderInfoNamespace as AppNavigationHeaderInfo };
